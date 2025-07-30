pub mod controllers;
pub mod graphql;
pub mod utils;

use std::sync::Arc;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{Router, extract::DefaultBodyLimit, routing::get};
use tokio::net::TcpListener;

use crate::{
    config::Config,
    database::IDatabase,
    http_server::{
        controllers::{collection, health, nft},
        graphql::{Query, graphql},
    },
    services::{IInternalServices, Services},
    utils::shutdown_utils,
};

pub struct HttpServer<TDb: IDatabase, TInternalService: IInternalServices> {
    db: Arc<TDb>,
    config: Arc<Config>,
    services: Arc<Services<TInternalService>>,
}

impl<TDb, TInternalService> HttpServer<TDb, TInternalService>
where
    TInternalService: IInternalServices + Send + 'static,
    TDb: IDatabase + Send + Sync + 'static,
{
    pub fn new(
        db: Arc<TDb>,
        config: Arc<Config>,
        services: Arc<Services<TInternalService>>,
    ) -> Self {
        Self {
            db,
            config,
            services,
        }
    }

    pub async fn start(self) -> anyhow::Result<()> {
        tracing::info!("Starting HTTP server...");

        let state = Arc::new(self);

        let listener_address = format!("0.0.0.0:{}", state.config.server_port);
        let listener = TcpListener::bind(listener_address).await?;

        axum::serve(listener, state.router())
            .with_graceful_shutdown(Self::shutdown_signal())
            .await
            .expect("HTTP server crashed");

        tracing::info!("HTTP server completed");

        Ok(())
    }

    fn router(self: &Arc<Self>) -> Router {
        let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
            .data(Arc::clone(&self.db))
            .finish();

        Router::new()
            .route("/health", get(health::check))
            .route("/gql", get(graphql).post_service(GraphQL::new(schema)))
            .nest(
                "/api/v1",
                Router::new()
                    .nest(
                        "/collections",
                        Router::new()
                            .route("/{id}/offers", get(collection::offers))
                            .route("/{id}/top-buyers", get(collection::top_buyers))
                            .route("/{id}/top-sellers", get(collection::top_sellers))
                            .route("/{id}/nft-holders", get(collection::nft_holders))
                            .route("/{id}/nft-trendings", get(collection::nft_trendings))
                            .route("/{id}/nft-change", get(collection::nft_change))
                            .route(
                                "/{id}/profit-leaderboard",
                                get(collection::profit_leaderboard),
                            )
                            .route(
                                "/{id}/nft-amount-distribution",
                                get(collection::nft_amount_distribution),
                            )
                            .route(
                                "/{id}/nft-period-distribution",
                                get(collection::nft_period_distribution),
                            ),
                    )
                    .nest(
                        "/nfts",
                        Router::new()
                            .route("/{id}/offers", get(nft::offers))
                            .route("/{id}/listings", get(nft::listings)),
                    ),
            )
            .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
            .with_state(Arc::clone(self))
    }

    async fn shutdown_signal() {
        let cancel_token = shutdown_utils::get_shutdown_token();
        cancel_token.cancelled().await;
    }
}
