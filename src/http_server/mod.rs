pub mod controllers;
pub mod graphql;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use axum::{Router, extract::DefaultBodyLimit, routing::get};
use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    cors::{self, CorsLayer},
    limit::RequestBodyLimitLayer,
};

use crate::http_server::controllers::api_key;
use crate::{
    cache::ICache,
    config::Config,
    database::IDatabase,
    http_server::{
        controllers::{graphql_handler, health},
        graphql::{Query, graphql},
    },
    utils::shutdown_utils,
};

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

        let listener_address = format!("0.0.0.0:{}", state.config.server_port);
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

        Router::new()
            .route("/health", get(health::check))
            .route(
                "/api-keys",
                get(api_key::fetch_api_keys)
                    .post(api_key::create_api_key)
                    .delete(api_key::remove_api_key),
            )
            .route("/gql", get(graphql).post(graphql_handler))
            .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
            .layer(RequestBodyLimitLayer::new(8 * 1024 * 1024))
            .layer(cors)
            .layer(governor)
            .with_state(Arc::clone(self))
    }

    async fn shutdown_signal() {
        let cancel_token = shutdown_utils::get_shutdown_token();
        cancel_token.cancelled().await;
    }
}
