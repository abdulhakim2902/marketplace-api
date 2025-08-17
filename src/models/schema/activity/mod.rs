pub mod profit_loss;

use crate::models::schema::{
    Date, OperatorSchema, OrderingType, collection::CollectionSchema, fetch_collection, fetch_nft,
    nft::NftSchema,
};
use async_graphql::{ComplexObject, Context, InputObject, SimpleObject};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct ActivitySchema {
    pub id: Uuid,
    #[graphql(name = "type")]
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereActivitySchema {
    #[graphql(name = "_or")]
    pub _or: Option<Box<WhereActivitySchema>>,
    #[graphql(name = "_and")]
    pub _and: Option<Box<WhereActivitySchema>>,
    #[graphql(name = "_not")]
    pub _not: Option<Box<WhereActivitySchema>>,
    pub id: Option<OperatorSchema<Uuid>>,
    #[graphql(name = "type")]
    pub tx_type: Option<OperatorSchema<String>>,
    pub tx_index: Option<OperatorSchema<i64>>,
    pub tx_id: Option<OperatorSchema<String>>,
    pub sender: Option<OperatorSchema<String>>,
    pub receiver: Option<OperatorSchema<String>>,
    pub price: Option<OperatorSchema<i64>>,
    pub usd_price: Option<OperatorSchema<BigDecimal>>,
    pub market_name: Option<OperatorSchema<String>>,
    pub market_contract_id: Option<OperatorSchema<String>>,
    pub nft_id: Option<OperatorSchema<Uuid>>,
    pub collection_id: Option<OperatorSchema<Uuid>>,
    pub block_time: Option<OperatorSchema<Date>>,
    pub block_height: Option<OperatorSchema<i64>>,
    pub amount: Option<OperatorSchema<i64>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct OrderByActivitySchema {
    pub id: Option<OrderingType>,
    #[graphql(name = "type")]
    pub tx_type: Option<OrderingType>,
    pub tx_index: Option<OrderingType>,
    pub tx_id: Option<OrderingType>,
    pub sender: Option<OrderingType>,
    pub receiver: Option<OrderingType>,
    pub price: Option<OrderingType>,
    pub usd_price: Option<OrderingType>,
    pub market_name: Option<OrderingType>,
    pub market_contract_id: Option<OrderingType>,
    pub nft_id: Option<OrderingType>,
    pub collection_id: Option<OrderingType>,
    pub block_time: Option<OrderingType>,
    pub block_height: Option<OrderingType>,
    pub amount: Option<OrderingType>,
}
