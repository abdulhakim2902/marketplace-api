pub mod profit_loss;

use crate::models::schema::{
    collection::CollectionSchema, fetch_collection, fetch_nft, nft::NftSchema,
};
use async_graphql::{ComplexObject, Context, Enum, InputObject, InputType, SimpleObject};
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
pub struct FilterActivitySchema {
    #[graphql(name = "where")]
    pub where_: Option<WhereActivitySchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
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
    // TODO: handle for date time
    pub block_time: Option<DateTime<Utc>>,
    pub block_height: Option<OperatorSchema<i64>>,
    pub amount: Option<OperatorSchema<i64>>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, InputObject)]
#[graphql(
    concrete(
        name = "OperatorSchemaTypeOutString",
        input_name = "OperatorSchemaTypeInString",
        params(String)
    ),
    concrete(
        name = "OperatorSchemaTypeOutInt64",
        input_name = "OperatorSchemaTypeInInt64",
        params(i64)
    ),
    concrete(
        name = "OperatorSchemaTypeOutBigDecimal",
        input_name = "OperatorSchemaTypeInBigDecimal",
        params(BigDecimal)
    ),
    concrete(
        name = "OperatorSchemaTypeOutUuid",
        input_name = "OperatorSchemaTypeInUuid",
        params(Uuid)
    )
    // concrete(
    //     name = "OperatorSchemaTypeOutDateTime",
    //     input_name = "OperatorSchemaTypeInDateTime",
    //     params(DateTime)
    // )
)]
pub struct OperatorSchema<T: InputType> {
    #[graphql(name = "_eq")]
    _eq: Option<T>,
    #[graphql(name = "_in")]
    _in: Option<Vec<T>>,
    #[graphql(name = "_gt")]
    _gt: Option<T>,
    #[graphql(name = "_gte")]
    _gte: Option<T>,
    #[graphql(name = "_lt")]
    _lt: Option<T>,
    #[graphql(name = "_lte")]
    _lte: Option<T>,
    #[graphql(name = "_nin")]
    _nin: Option<Vec<T>>,
    #[graphql(name = "_neq")]
    _neq: Option<T>,
    #[graphql(name = "_is_null")]
    _is_null: Option<bool>,
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
