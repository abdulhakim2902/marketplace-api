pub mod profit_loss;

use crate::models::schema::{
    collection::CollectionSchema, fetch_collection, fetch_nft, nft::NftSchema,
};
use async_graphql::{ComplexObject, Context, Enum, InputObject, SimpleObject};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct ActivitySchema {
    pub id: Uuid,
    pub tx_type: Option<String>,
    pub tx_index: i64,
    pub tx_id: String,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub price: Option<i64>,
    pub usd_price: Option<BigDecimal>,
    pub market_name: Option<String>,
    pub market_contract_id: Option<String>,
    pub nft_id: Option<Uuid>,
    pub collection_id: Option<Uuid>,
    pub block_time: Option<DateTime<Utc>>,
    pub block_height: Option<i64>,
    pub amount: Option<i64>,
}

#[ComplexObject]
impl ActivitySchema {
    async fn nft(&self, ctx: &Context<'_>) -> Option<NftSchema> {
        fetch_nft(
            ctx,
            self.nft_id.as_ref().map(|e| e.to_string()),
            self.collection_id.as_ref().map(|e| e.to_string()),
        )
        .await
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        fetch_collection(ctx, self.collection_id.as_ref().map(|e| e.to_string())).await
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterActivitySchema {
    #[graphql(name = "where")]
    pub where_: Option<WhereActivitySchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereActivitySchema {
    pub id: Option<String>,
    pub wallet_address: Option<String>,
    pub collection_id: Option<String>,
    pub nft_id: Option<String>,
    pub tx_types: Option<Vec<TxType>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[graphql(rename_items = "snake_case")]
pub enum TxType {
    Sales,
    Offers,
    Listings,
    Transfers,
    Mints,
}
