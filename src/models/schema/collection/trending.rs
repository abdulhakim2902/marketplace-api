use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, SimpleObject, dataloader::DataLoader};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{
    database::{Database, IDatabase, collections::Collections},
    models::schema::collection::CollectionSchema,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct CollectionTrendingSchema {
    pub collection_id: Uuid,
    pub floor: Option<i64>,
    pub owners: Option<i64>,
    pub listed: Option<i64>,
    pub supply: Option<i64>,
    pub volume: Option<i64>,
    pub volume_usd: Option<BigDecimal>,
    pub sales: Option<i64>,
    pub market_cap: Option<i64>,
}

#[ComplexObject]
impl CollectionTrendingSchema {
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
