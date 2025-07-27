pub mod controllers;
pub mod utils;

use std::sync::Arc;

use axum::{Router, extract::DefaultBodyLimit, routing::get};
use tokio::net::TcpListener;

use crate::{
    config::Config,
    http_server::controllers::{collection, health, nft},
    services::{IInternalServices, Services},
    utils::shutdown_utils,
};

pub struct HttpServer<TInternalService: IInternalServices> {
    config: Arc<Config>,
    services: Arc<Services<TInternalService>>,
}

impl<TInternalService> HttpServer<TInternalService>
where
    TInternalService: IInternalServices + Send + 'static,
{
    pub fn new(config: Arc<Config>, services: Arc<Services<TInternalService>>) -> Self {
        Self { config, services }
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
        Router::new()
            .route("/health", get(health::check))
            .nest(
                "/api/v1",
                Router::new()
                    .nest(
                        "/collections",
                        Router::new()
                            .route("/", get(collection::filter))
                            .route("/{id}", get(collection::info))
                            .route("/{id}/nfts", get(collection::nfts))
                            .route("/{id}/offers", get(collection::offers))
                            .route("/{id}/activities", get(collection::activities)),
                    )
                    .nest(
                        "/nfts",
                        Router::new()
                            .route("/{id}/activities", get(nft::activities))
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
