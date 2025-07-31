pub mod activities;
pub mod attributes;
pub mod bids;
pub mod collections;
pub mod listings;
pub mod nfts;
pub mod processor_status;
pub mod token_prices;
pub mod wallets;

use std::sync::Arc;

use sqlx::{Pool, Postgres, migrate::Migrator};

use crate::database::{
    activities::{Activities, IActivities},
    attributes::{Attributes, IAttributes},
    bids::{Bids, IBids},
    collections::{Collections, ICollections},
    listings::{IListings, Listings},
    nfts::{INfts, Nfts},
    processor_status::{IProcessorStatus, ProcessorStatus},
    token_prices::{ITokenPrices, TokenPrices},
    wallets::{IWallets, Wallets},
};

#[async_trait::async_trait]
pub trait IDatabase: Send + Sync + 'static {
    type TTActivities: IActivities;
    type TTBids: IBids;
    type TTListings: IListings;
    type TTCollections: ICollections;
    type TTNfts: INfts;
    type TTAttributes: IAttributes;
    type TTokenPrices: ITokenPrices;
    type TProcessorStatus: IProcessorStatus;
    type TWallets: IWallets;

    async fn is_healthy(&self) -> bool;

    fn get_pool(&self) -> &Pool<Postgres>;
    fn activities(&self) -> Arc<Self::TTActivities>;
    fn bids(&self) -> Arc<Self::TTBids>;
    fn listings(&self) -> Arc<Self::TTListings>;
    fn collections(&self) -> Arc<Self::TTCollections>;
    fn nfts(&self) -> Arc<Self::TTNfts>;
    fn attributes(&self) -> Arc<Self::TTAttributes>;
    fn wallets(&self) -> Arc<Self::TWallets>;
    fn token_prices(&self) -> Arc<Self::TTokenPrices>;
    fn processor_status(&self) -> Arc<Self::TProcessorStatus>;
}

pub struct Database {
    pool: Arc<Pool<Postgres>>,
    activities: Arc<Activities>,
    bids: Arc<Bids>,
    listings: Arc<Listings>,
    collections: Arc<Collections>,
    nfts: Arc<Nfts>,
    attributes: Arc<Attributes>,
    wallets: Arc<Wallets>,
    token_prices: Arc<TokenPrices>,
    processor_status: Arc<ProcessorStatus>,
}

impl Database {
    pub fn new(
        pool: Arc<Pool<Postgres>>,
        activities: Arc<Activities>,
        bids: Arc<Bids>,
        listings: Arc<Listings>,
        collections: Arc<Collections>,
        nfts: Arc<Nfts>,
        attributes: Arc<Attributes>,
        token_prices: Arc<TokenPrices>,
        wallets: Arc<Wallets>,
        processor_status: Arc<ProcessorStatus>,
    ) -> Self {
        Self {
            pool,
            activities,
            bids,
            listings,
            collections,
            nfts,
            attributes,
            token_prices,
            wallets,
            processor_status,
        }
    }

    pub async fn migrate(pool: &Pool<Postgres>) -> anyhow::Result<()> {
        let migrator = Migrator::new(std::path::Path::new("./migrations")).await?;

        migrator.run(pool).await?;

        tracing::info!(
            "Database migrations finished. Version: {}",
            migrator.migrations.last().map(|m| m.version).unwrap_or(0)
        );

        Ok(())
    }
}

#[async_trait::async_trait]
impl IDatabase for Database {
    type TTActivities = Activities;
    type TTBids = Bids;
    type TTListings = Listings;
    type TTCollections = Collections;
    type TTNfts = Nfts;
    type TTAttributes = Attributes;
    type TProcessorStatus = ProcessorStatus;
    type TTokenPrices = TokenPrices;
    type TWallets = Wallets;

    async fn is_healthy(&self) -> bool {
        sqlx::query("SELECT 1").fetch_one(&*self.pool).await.is_ok()
    }

    fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    fn activities(&self) -> Arc<Self::TTActivities> {
        Arc::clone(&self.activities)
    }

    fn bids(&self) -> Arc<Self::TTBids> {
        Arc::clone(&self.bids)
    }

    fn listings(&self) -> Arc<Self::TTListings> {
        Arc::clone(&self.listings)
    }

    fn collections(&self) -> Arc<Self::TTCollections> {
        Arc::clone(&self.collections)
    }

    fn nfts(&self) -> Arc<Self::TTNfts> {
        Arc::clone(&self.nfts)
    }

    fn attributes(&self) -> Arc<Self::TTAttributes> {
        Arc::clone(&self.attributes)
    }

    fn token_prices(&self) -> Arc<Self::TTokenPrices> {
        Arc::clone(&self.token_prices)
    }

    fn wallets(&self) -> Arc<Self::TWallets> {
        Arc::clone(&self.wallets)
    }

    fn processor_status(&self) -> Arc<Self::TProcessorStatus> {
        Arc::clone(&self.processor_status)
    }
}
