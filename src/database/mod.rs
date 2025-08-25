pub mod activities;
pub mod api_keys;
pub mod attributes;
pub mod bids;
pub mod collections;
pub mod listings;
pub mod marketplaces;
pub mod nft_metadata;
pub mod nfts;
pub mod processor_status;
pub mod request_logs;
pub mod token_prices;
pub mod users;
pub mod wallets;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, migrate::Migrator};
use strum::{Display, EnumString};

use crate::database::{
    activities::{Activities, IActivities},
    api_keys::{ApiKeys, IApiKeys},
    attributes::{Attributes, IAttributes},
    bids::{Bids, IBids},
    collections::{Collections, ICollections},
    listings::{IListings, Listings},
    marketplaces::{IMarketplaces, Marketplaces},
    nft_metadata::{INFTMetadata, NFTMetadata},
    nfts::{INfts, Nfts},
    processor_status::{IProcessorStatus, ProcessorStatus},
    request_logs::{IRequestLogs, RequestLogs},
    token_prices::{ITokenPrices, TokenPrices},
    users::{IUsers, Users},
    wallets::{IWallets, Wallets},
};

#[async_trait::async_trait]
pub trait IDatabase: Send + Sync + 'static {
    type TActivities: IActivities;
    type TBids: IBids;
    type TListings: IListings;
    type TCollections: ICollections;
    type TNfts: INfts;
    type TAttributes: IAttributes;
    type TokenPrices: ITokenPrices;
    type TProcessorStatus: IProcessorStatus;
    type TWallets: IWallets;
    type TMarketplaces: IMarketplaces;
    type TNFTMetadata: INFTMetadata;
    type TUsers: IUsers;
    type TRequestLogs: IRequestLogs;
    type TApiKeys: IApiKeys;

    async fn is_healthy(&self) -> bool;

    fn get_pool(&self) -> &Pool<Postgres>;
    fn activities(&self) -> Arc<Self::TActivities>;
    fn bids(&self) -> Arc<Self::TBids>;
    fn listings(&self) -> Arc<Self::TListings>;
    fn collections(&self) -> Arc<Self::TCollections>;
    fn nfts(&self) -> Arc<Self::TNfts>;
    fn attributes(&self) -> Arc<Self::TAttributes>;
    fn wallets(&self) -> Arc<Self::TWallets>;
    fn token_prices(&self) -> Arc<Self::TokenPrices>;
    fn marketplaces(&self) -> Arc<Self::TMarketplaces>;
    fn nft_metadata(&self) -> Arc<Self::TNFTMetadata>;
    fn processor_status(&self) -> Arc<Self::TProcessorStatus>;
    fn users(&self) -> Arc<Self::TUsers>;
    fn request_logs(&self) -> Arc<Self::TRequestLogs>;
    fn api_keys(&self) -> Arc<Self::TApiKeys>;
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
    marketplaces: Arc<Marketplaces>,
    nft_metadata: Arc<NFTMetadata>,
    users: Arc<Users>,
    request_logs: Arc<RequestLogs>,
    api_keys: Arc<ApiKeys>,
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
        marketplaces: Arc<Marketplaces>,
        nft_metadata: Arc<NFTMetadata>,
        users: Arc<Users>,
        request_logs: Arc<RequestLogs>,
        api_keys: Arc<ApiKeys>,
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
            marketplaces,
            nft_metadata,
            users,
            request_logs,
            api_keys,
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
    type TActivities = Activities;
    type TBids = Bids;
    type TListings = Listings;
    type TCollections = Collections;
    type TNfts = Nfts;
    type TAttributes = Attributes;
    type TProcessorStatus = ProcessorStatus;
    type TokenPrices = TokenPrices;
    type TWallets = Wallets;
    type TMarketplaces = Marketplaces;
    type TNFTMetadata = NFTMetadata;
    type TUsers = Users;
    type TRequestLogs = RequestLogs;
    type TApiKeys = ApiKeys;

    async fn is_healthy(&self) -> bool {
        sqlx::query("SELECT 1").fetch_one(&*self.pool).await.is_ok()
    }

    fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    fn activities(&self) -> Arc<Self::TActivities> {
        Arc::clone(&self.activities)
    }

    fn bids(&self) -> Arc<Self::TBids> {
        Arc::clone(&self.bids)
    }

    fn listings(&self) -> Arc<Self::TListings> {
        Arc::clone(&self.listings)
    }

    fn collections(&self) -> Arc<Self::TCollections> {
        Arc::clone(&self.collections)
    }

    fn nfts(&self) -> Arc<Self::TNfts> {
        Arc::clone(&self.nfts)
    }

    fn attributes(&self) -> Arc<Self::TAttributes> {
        Arc::clone(&self.attributes)
    }

    fn token_prices(&self) -> Arc<Self::TokenPrices> {
        Arc::clone(&self.token_prices)
    }

    fn wallets(&self) -> Arc<Self::TWallets> {
        Arc::clone(&self.wallets)
    }

    fn processor_status(&self) -> Arc<Self::TProcessorStatus> {
        Arc::clone(&self.processor_status)
    }

    fn marketplaces(&self) -> Arc<Self::TMarketplaces> {
        Arc::clone(&self.marketplaces)
    }

    fn nft_metadata(&self) -> Arc<Self::TNFTMetadata> {
        Arc::clone(&self.nft_metadata)
    }

    fn users(&self) -> Arc<Self::TUsers> {
        Arc::clone(&self.users)
    }

    fn request_logs(&self) -> Arc<Self::TRequestLogs> {
        Arc::clone(&self.request_logs)
    }

    fn api_keys(&self) -> Arc<Self::TApiKeys> {
        Arc::clone(&self.api_keys)
    }
}

#[derive(Debug, Clone, EnumString, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Schema {
    Activities,
    Attributes,
    Bids,
    Nfts,
    Collections,
    Listings,
    Wallets,
}
