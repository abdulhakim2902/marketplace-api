use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, SimpleObject, dataloader::DataLoader};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};
use uuid::Uuid;

use crate::{
    database::{Database, IDatabase, collections::Collections},
    models::schema::{collection::CollectionSchema, fetch_token_price},
};

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(complex, name = "CollectionTrending", rename_fields = "snake_case")]
pub struct CollectionTrendingSchema {
    pub collection_id: Uuid,
    pub market_cap: Option<i64>,
    pub floor: Option<i64>,
    pub floor_percentage: Option<BigDecimal>,
    pub listed: Option<i64>,
    pub listed_percentage: Option<BigDecimal>,
    pub volume: Option<i64>,
    pub volume_percentage: Option<BigDecimal>,
    pub sales: Option<i64>,
    pub owners: Option<i64>,
    pub owners_percentage: Option<BigDecimal>,
    pub top_bid: Option<i64>,
    pub total_volume: Option<i64>,
}

#[ComplexObject]
impl CollectionTrendingSchema {
    async fn apt_price(&self, ctx: &Context<'_>) -> Option<BigDecimal> {
        fetch_token_price(ctx).await
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let data_loader = DataLoader::new(
            Collections::new(Arc::new(db.get_pool().clone())),
            tokio::spawn,
        );

        data_loader
            .load_one(self.collection_id)
            .await
            .ok()
            .flatten()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[graphql(rename_items = "snake_case")]
pub enum OrderTrendingType {
    MarketCap,
    Floor,
    FloorPercentage,
    Listed,
    ListedPercentage,
    Volume,
    VolumePercentage,
    Sales,
    Owners,
    OwnersPercentage,
    TopBid,
    TotalVolume,
}

impl Default for OrderTrendingType {
    fn default() -> Self {
        Self::TotalVolume
    }
}
