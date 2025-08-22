use crate::{
    database::{Database, IDatabase, bids::IBids, token_prices::ITokenPrices},
    models::schema::{
        activity::ActivitySchema, attribute::AttributeSchema, bid::BidSchema,
        collection::CollectionSchema, listing::ListingSchema, nft::NftSchema,
    },
};
use async_graphql::{Context, Enum, InputObject, InputType, OutputType, SimpleObject};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::{Display, EnumString};
use uuid::Uuid;

pub mod activity;
pub mod attribute;
pub mod bid;
pub mod collection;
pub mod data_point;
pub mod listing;
pub mod marketplace;
pub mod nft;
pub mod wallet;

pub type Date = DateTime<Utc>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[graphql(name = "Order", rename_items = "snake_case")]
pub enum OrderingType {
    Asc,
    AscNullsFirst,
    AscNullsLast,
    Desc,
    DescNullsLast,
    DescNullsFirst,
}

impl Default for OrderingType {
    fn default() -> Self {
        Self::Desc
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[graphql(name = "Coin", rename_items = "snake_case")]
pub enum CoinType {
    Apt,
    Usd,
}

impl Default for CoinType {
    fn default() -> Self {
        Self::Apt
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, InputObject)]
#[graphql(
    name = "ComparisonOperator",
    concrete(
        name = "OperatorTypeOutString",
        input_name = "OperatorTypeInString",
        params(String)
    ),
    concrete(
        name = "OperatorTypeOutInt64",
        input_name = "OperatorTypeInInt64",
        params(i64)
    ),
    concrete(
        name = "OperatorTypeOutBigDecimal",
        input_name = "OperatorTypeInBigDecimal",
        params(BigDecimal)
    ),
    concrete(
        name = "OperatorTypeOutUuid",
        input_name = "OperatorTypeInUuid",
        params(Uuid)
    ),
    concrete(
        name = "OperatorTypeOutDateTime",
        input_name = "OperatorTypeInDateTime",
        params(Date)
    ),
    concrete(
        name = "OperatorTypeOutBool",
        input_name = "OperatorTypeInBool",
        params(bool)
    )
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

#[derive(Serialize, Deserialize, Default, Clone, Debug, SimpleObject)]
#[graphql(name = "AggregateFields", rename_fields = "snake_case")]
pub struct AggregateFieldsSchema {
    pub count: i64,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, SimpleObject)]
#[graphql(
    name = "Aggregate",
    rename_fields = "snake_case",
    concrete(
        name = "AggregateTypeOutActivity",
        input_name = "AggregateTypeInActivity",
        params(ActivitySchema)
    ),
    concrete(
        name = "AggregateTypeOutAttribute",
        input_name = "AggregateTypeInAttribute",
        params(AttributeSchema)
    ),
    concrete(
        name = "AggregateTypeOutCollection",
        input_name = "AggregateTypeInCollection",
        params(CollectionSchema)
    ),
    concrete(
        name = "AggregateTypeOutNft",
        input_name = "AggregateTypeInNft",
        params(NftSchema)
    ),
    concrete(
        name = "AggregateTypeOutListing",
        input_name = "AggregateTypeInListing",
        params(ListingSchema)
    ),
    concrete(
        name = "AggregateTypeOutBid",
        input_name = "AggregateTypeInBid",
        params(BidSchema)
    )
)]
pub struct AggregateSchema<T: OutputType> {
    pub aggregate: AggregateFieldsSchema,
    pub nodes: Vec<T>,
}

impl<T: OutputType> AggregateSchema<T> {
    pub fn new(total: i64, nodes: Vec<T>) -> Self {
        Self {
            aggregate: AggregateFieldsSchema { count: total },
            nodes,
        }
    }
}

async fn fetch_total_collection_offer(
    ctx: &Context<'_>,
    collection_id: Option<String>,
) -> Option<String> {
    if collection_id.is_none() {
        return None;
    }

    let collection_id = collection_id.as_ref().unwrap();

    let db = ctx
        .data::<Arc<Database>>()
        .expect("Missing database in the context");

    let res = db.bids().fetch_total_collection_offer(collection_id).await;

    if res.is_err() {
        return None;
    }

    res.unwrap().as_ref().map(|e| e.to_plain_string())
}

async fn fetch_token_price(ctx: &Context<'_>) -> Option<BigDecimal> {
    let db = ctx
        .data::<Arc<Database>>()
        .expect("Missing database in the context");

    db.token_prices()
        .fetch_token_price("0x000000000000000000000000000000000000000000000000000000000000000a")
        .await
        .ok()
}
