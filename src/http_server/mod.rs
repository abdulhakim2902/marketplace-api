pub mod controllers;
pub mod graphql;
pub mod middlewares;
pub mod utils;

use crate::{
    cache::ICache,
    config::Config,
    database::IDatabase,
    http_server::{
        controllers::{
            api_key::{self, API_KEY_TAG},
            auth::{self, AUTH_TAG},
            graphql_handler, health,
            user::{self, USER_TAG},
        },
        graphql::{Query, graphql},
        middlewares::{authentication, authorize},
    },
    utils::shutdown_utils,
};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, patch, post},
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    compression::{CompressionLayer, CompressionLevel},
    cors::{self, CorsLayer},
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::{RequestBodyTimeoutLayer, TimeoutLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(paths(auth::login))]
struct AuthApi;

#[derive(OpenApi)]
#[openapi(paths(user::fetch_user, user::create_user, user::update_user))]
struct UserApi;

#[derive(OpenApi)]
#[openapi(paths(
    api_key::fetch_api_keys,
    api_key::create_api_key,
    api_key::update_api_key,
    api_key::remove_api_key
))]
struct ApiKeyApi;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/auth", api = AuthApi),
        (path = "/users", api = UserApi),
        (path = "/api-keys", api = ApiKeyApi),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = AUTH_TAG, description = "Auth items management API"),
        (name = USER_TAG, description = "User items management API"),
        (name = API_KEY_TAG, description = "User api key items management API")
    ),
    servers((url = "/api/v1"))
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "BearerAuth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
        }
    }
}

pub struct HttpServer<TDb: IDatabase, TCache: ICache> {
    db: Arc<TDb>,
    _cache: Arc<TCache>,
    config: Arc<Config>,
    schema: Arc<Schema<Query, EmptyMutation, EmptySubscription>>,
}

impl<TDb, TCache> HttpServer<TDb, TCache>
where
    TDb: IDatabase + Send + Sync + 'static,
    TCache: ICache + 'static,
{
    pub fn new(db: Arc<TDb>, _cache: Arc<TCache>, config: Arc<Config>) -> Self {
        let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
            .data(Arc::clone(&db))
            .finish();

        Self {
            db,
            _cache,
            config,
            schema: Arc::new(schema),
        }
    }

    pub async fn start(self) -> anyhow::Result<()> {
        tracing::info!("Starting HTTP server...");

        let state = Arc::new(self);

        let listener_address = format!("0.0.0.0:{}", state.config.server_config.port);
        let listener = TcpListener::bind(listener_address).await?;

        axum::serve(
            listener,
            state
                .router()
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(Self::shutdown_signal())
        .await
        .expect("HTTP server crashed");

        tracing::info!("HTTP server completed");

        Ok(())
    }

    fn router(self: &Arc<Self>) -> Router {
        let api_middleware = ServiceBuilder::new()
            .layer(CompressionLayer::new().quality(CompressionLevel::Fastest))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(
                        DefaultMakeSpan::new()
                            .level(Level::INFO)
                            .include_headers(true),
                    )
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .include_headers(true),
                    ),
            )
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(RequestBodyTimeoutLayer::new(Duration::from_secs(4)))
            .layer(TimeoutLayer::new(Duration::from_secs(5)));

        let governor_config = GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap();

        let governor_limiter = governor_config.limiter().clone();
        let interval = Duration::from_secs(60);

        let cors = CorsLayer::new()
            .allow_headers(cors::Any)
            .allow_methods(cors::Any)
            .expose_headers(cors::Any)
            .max_age(Duration::from_secs(24 * 3600));

        let governor = GovernorLayer::new(governor_config);

        // a separate background task to clean up
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(interval);
                tracing::info!("rate limiting storage size: {}", governor_limiter.len());
                governor_limiter.retain_recent();
            }
        });

        let db = Arc::clone(&self.db);
        let jwt_secret = self.config.jwt_config.secret.to_string();

        let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
            .route("/health", get(health::check))
            .nest(
                "/api/v1",
                OpenApiRouter::new()
                    .nest(
                        "/users",
                        OpenApiRouter::new()
                            .route("/", get(user::fetch_user).post(user::create_user))
                            .route("/{id}", patch(user::update_user))
                            .layer(middleware::from_fn(authorize::authorize_admin)),
                    )
                    .nest(
                        "/api-keys",
                        OpenApiRouter::new()
                            .route(
                                "/",
                                get(api_key::fetch_api_keys).post(api_key::create_api_key),
                            )
                            .route(
                                "/{id}",
                                delete(api_key::remove_api_key).patch(api_key::update_api_key),
                            )
                            .layer(middleware::from_fn(authorize::authorize_user)),
                    )
                    .layer(middleware::from_fn(move |req, next| {
                        authentication::authentication(
                            req,
                            next,
                            Arc::clone(&db),
                            jwt_secret.clone(),
                        )
                    }))
                    .nest(
                        "/auth",
                        OpenApiRouter::new().route("/login", post(auth::login)),
                    )
                    .layer(api_middleware),
            )
            .route("/gql", get(graphql).post(graphql_handler))
            .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
            .layer(RequestBodyLimitLayer::new(8 * 1024 * 1024))
            .layer(cors)
            .layer(governor)
            .with_state(Arc::clone(self))
            .split_for_parts();

        router.merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", api.clone()))
    }

    async fn shutdown_signal() {
        let cancel_token = shutdown_utils::get_shutdown_token();
        cancel_token.cancelled().await;
    }
}
