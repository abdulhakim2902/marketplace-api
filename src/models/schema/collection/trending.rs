use async_graphql::{Enum, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CollectionTrendingSchema {
    pub id: Uuid,
    pub floor: Option<i64>,
    pub owners: Option<i64>,
    pub listed: Option<i64>,
    pub supply: Option<i64>,
    pub volume: Option<i64>,
    pub volume_usd: Option<BigDecimal>,
    pub sales: Option<i64>,
    pub market_cap: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
#[graphql(rename_items = "snake_case")]
pub enum OrderTrendingType {
    Volume,
    Floor,
    Owners,
    MarketCap,
    Sales,
    Listed,
}

impl Default for OrderTrendingType {
    fn default() -> Self {
        Self::Volume
    }
}
