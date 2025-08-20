pub mod cache;
pub mod config;
pub mod database;
pub mod http_server;
pub mod models;
pub mod utils;
pub mod workers;

use anyhow::Context;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use std::{sync::Arc, time::Duration};
use tracing_subscriber::{EnvFilter, prelude::*};

use crate::{
    cache::Cache,
    config::Config,
    database::{
        Database, IDatabase,
        activities::Activities,
        api_keys::ApiKeys,
        attributes::Attributes,
        bids::Bids,
        collections::Collections,
        listings::Listings,
        marketplaces::Marketplaces,
        nft_metadata::NFTMetadata,
        nfts::Nfts,
        processor_status::ProcessorStatus,
        request_logs::RequestLogs,
        token_prices::TokenPrices,
        users::{IUsers, Users},
        wallets::Wallets,
    },
    http_server::HttpServer,
    utils::shutdown_utils,
    workers::{Worker, price_indexer::PriceIndexer},
};

pub async fn init() -> anyhow::Result<(Arc<Worker<Database, Cache>>, HttpServer<Database, Cache>)> {
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
        Arc::new(Wallets::new(Arc::clone(&pool))),
        Arc::new(ProcessorStatus::new(Arc::clone(&pool))),
        Arc::new(Marketplaces::new(Arc::clone(&pool))),
        Arc::new(NFTMetadata::new(Arc::clone(&pool))),
        Arc::new(Users::new(Arc::clone(&pool))),
        Arc::new(RequestLogs::new(Arc::clone(&pool))),
        Arc::new(ApiKeys::new(Arc::clone(&pool))),
    ));

    init_admin(
        Arc::clone(&db),
        &config.admin_config.user,
        &config.admin_config.password,
    )
    .await
    .context("Failed to initialize admin")?;

    init_price(&config.tapp_url, Arc::clone(&db), Arc::clone(&cache))
        .await
        .context("Failed to initialize price")?;

    tokio::spawn(shutdown_utils::poll_for_shutdown_signal());

    Ok((
        Arc::new(Worker::new(
            Arc::clone(&config),
            Arc::clone(&db),
            Arc::clone(&cache),
        )),
        HttpServer::new(Arc::clone(&db), Arc::clone(&cache), Arc::clone(&config)),
    ))
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

async fn init_price(tapp_url: &str, db: Arc<Database>, cache: Arc<Cache>) -> anyhow::Result<()> {
    let client = Client::new();
    let price_indexer = PriceIndexer::new(tapp_url.to_string(), db, cache);

    price_indexer.fetch_and_store_token_prices(&client).await?;

    Ok(())
}

async fn init_admin(db: Arc<Database>, user: &str, password: &str) -> anyhow::Result<()> {
    db.users().clean_admin_user().await?;
    db.users().create_admin_user(&user, password).await?;

    Ok(())
}
