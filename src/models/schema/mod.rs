use crate::{
    database::{Database, IDatabase, token_prices::ITokenPrices},
    models::schema::{
        activity::{ActivitySchema, AggregateActivityFieldsSchema, AggregateActivitySchema},
        attribute::{AggregateAttributeFieldsSchema, AggregateAttributeSchema, AttributeSchema},
        bid::{AggregateBidFieldsSchema, AggregateBidSchema, BidSchema},
        collection::{
            AggregateCollectionFieldsSchema, AggregateCollectionSchema, CollectionSchema,
        },
        listing::{AggregateListingFieldsSchema, AggregateListingSchema, ListingSchema},
        nft::{AggregateNftFieldsSchema, AggregateNftSchema, NftSchema},
    },
};
use async_graphql::{Context, Enum, InputObject, InputType, OutputType, SimpleObject};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
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
#[graphql(
    name = "AggregateFields",
    rename_fields = "snake_case",
    concrete(
        name = "AggregateFieldsTypeOutActivity",
        input_name = "AggregateFieldsTypeInActivity",
        params(AggregateActivityFieldsSchema)
    ),
    concrete(
        name = "AggregateFieldsTypeOutAttribute",
        input_name = "AggregateFieldsTypeInAttribute",
        params(AggregateAttributeFieldsSchema)
    ),
    concrete(
        name = "AggregateFieldsTypeOutBid",
        input_name = "AggregateFieldsTypeInBid",
        params(AggregateBidFieldsSchema)
    ),
    concrete(
        name = "AggregateFieldsTypeOutCollection",
        input_name = "AggregateFieldsTypeInCollection",
        params(AggregateCollectionFieldsSchema)
    ),
    concrete(
        name = "AggregateFieldsTypeOutListing",
        input_name = "AggregateFieldsTypeInListing",
        params(AggregateListingFieldsSchema)
    ),
    concrete(
        name = "AggregateFieldsTypeOutNft",
        input_name = "AggregateFieldsTypeInNft",
        params(AggregateNftFieldsSchema)
    )
)]
pub struct AggregateFieldsSchema<T: OutputType> {
    pub count: Option<i64>,
    pub avg: Option<T>,
    pub max: Option<T>,
    pub min: Option<T>,
    pub stddev: Option<T>,
    pub stddev_pop: Option<T>,
    pub sum: Option<T>,
    pub var_pop: Option<T>,
    pub var_samp: Option<T>,
    pub variance: Option<T>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, SimpleObject)]
#[graphql(
    name = "Aggregate",
    rename_fields = "snake_case",
    concrete(
        name = "AggregateTypeOutActivity",
        input_name = "AggregateTypeInActivity",
        params(AggregateActivitySchema, ActivitySchema)
    ),
    concrete(
        name = "AggregateTypeOutAttribute",
        input_name = "AggregateTypeInAttribute",
        params(AggregateAttributeSchema, AttributeSchema)
    ),
    concrete(
        name = "AggregateTypeOutBid",
        input_name = "AggregateTypeInBid",
        params(AggregateBidSchema, BidSchema)
    ),
    concrete(
        name = "AggregateTypeOutCollection",
        input_name = "AggregateTypeInCollection",
        params(AggregateCollectionSchema, CollectionSchema)
    ),
    concrete(
        name = "AggregateTypeOutListing",
        input_name = "AggregateTypeInListing",
        params(AggregateListingSchema, ListingSchema)
    ),
    concrete(
        name = "AggregateTypeOutNft",
        input_name = "AggregateTypeInNft",
        params(AggregateNftSchema, NftSchema)
    )
)]
pub struct AggregateSchema<T: OutputType, U: OutputType> {
    pub aggregate: T,
    pub nodes: Vec<U>,
}

#[derive(Clone, Debug)]
pub struct AggregateSelection {
    pub nodes: HashMap<String, Vec<String>>,
    pub aggregate: HashMap<String, Vec<String>>,
}

pub fn get_aggregate_selection(ctx: &Context<'_>) -> AggregateSelection {
    let mut nodes = HashMap::new();
    let mut aggregate = HashMap::new();

    for field in ctx.field().selection_set() {
        if field.name() == "aggregate" {
            for field in field.selection_set() {
                aggregate.insert(
                    field.name().to_string(),
                    field
                        .selection_set()
                        .map(|e| e.name().to_string())
                        .collect::<Vec<String>>(),
                );
            }
        } else if field.name() == "nodes" {
            for field in field.selection_set() {
                nodes.insert(
                    field.name().to_string(),
                    field
                        .selection_set()
                        .map(|e| e.name().to_string())
                        .collect::<Vec<String>>(),
                );
            }
        }
    }

    AggregateSelection { nodes, aggregate }
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
