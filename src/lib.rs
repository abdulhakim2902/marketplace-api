pub mod cache;
pub mod config;
pub mod database;
pub mod http_server;
pub mod models;
pub mod services;
pub mod utils;
pub mod workers;

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use std::{sync::Arc, time::Duration};
use tracing_subscriber::{EnvFilter, prelude::*};

use crate::{
    cache::Cache,
    config::Config,
    database::{
        Database, activities::Activities, attributes::Attributes, bids::Bids,
        collections::Collections, listings::Listings, nfts::Nfts,
        processor_status::ProcessorStatus, token_prices::TokenPrices,
    },
    http_server::HttpServer,
    services::{InternalServices, Services, collection::CollectionService, health::HealthService},
    utils::shutdown_utils,
    workers::Worker,
};

pub async fn init() -> anyhow::Result<(Arc<Worker<Database, Cache>>, HttpServer<InternalServices>)>
{
    let config = Arc::new(init_config().context("Failed to initialize configuration")?);
    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(config.db_config.pool_size)
            .acquire_timeout(Duration::from_secs(5 * 60))
            .connect(&config.db_config.url)
            .await?,
    );

    Database::migrate(&pool)
        .await
        .context("Failed to run migrations.")?;

    let cache = Arc::new(Cache::default());

    let db = Arc::new(Database::new(
        Arc::clone(&pool),
        Arc::new(Activities::new(Arc::clone(&pool))),
        Arc::new(Bids::new(Arc::clone(&pool))),
        Arc::new(Listings::new(Arc::clone(&pool))),
        Arc::new(Collections::new(Arc::clone(&pool))),
        Arc::new(Nfts::new(Arc::clone(&pool))),
        Arc::new(Attributes::new(Arc::clone(&pool))),
        Arc::new(TokenPrices::new(Arc::clone(&pool))),
        Arc::new(ProcessorStatus::new(Arc::clone(&pool))),
    ));

    let services = Arc::new(init_services(Arc::clone(&db)));

    tokio::spawn(shutdown_utils::poll_for_shutdown_signal());

    Ok((
        Arc::new(Worker::new(
            Arc::clone(&config),
            Arc::clone(&db),
            Arc::clone(&cache),
        )),
        HttpServer::new(Arc::clone(&config), Arc::clone(&services)),
    ))
}

fn init_services(db: Arc<Database>) -> Arc<Services<InternalServices>> {
    let health_service = Arc::new(HealthService::new(Arc::clone(&db)));
    let collection_service = Arc::new(CollectionService::new(Arc::clone(&db)));

    Arc::new(Services::new(health_service, collection_service))
}

fn init_config() -> anyhow::Result<Config> {
    let config = Config::load().context("Failed to load configuration")?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(EnvFilter::from_default_env())
        .init();

    tracing::trace!("config: {:#?}", config);

    Ok(config)
}
