use crate::{
    database::{
        Database, IDatabase, attributes::IAttributes, bids::IBids, nfts::INfts,
        token_prices::ITokenPrices,
    },
    models::schema::nft::{FilterNftSchema, NftSchema, WhereNftSchema},
};
use async_graphql::{Context, Enum, InputObject, InputType, OutputType};
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
#[graphql(rename_items = "snake_case")]
pub enum OrderingType {
    ASC,
    DESC,
}

impl Default for OrderingType {
    fn default() -> Self {
        Self::DESC
    }
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
    ),
    concrete(
        name = "OperatorSchemaTypeOutDateTime",
        input_name = "OperatorSchemaTypeInDateTime",
        params(Date)
    ),
    concrete(
        name = "OperatorSchemaTypeOutBool",
        input_name = "OperatorSchemaTypeInBool",
        params(bool)
    )
)]
pub struct OperatorSchema<T: InputType + OutputType> {
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

async fn fetch_nft(
    ctx: &Context<'_>,
    nft_id: Option<String>,
    collection_id: Option<String>,
) -> Option<NftSchema> {
    if nft_id.is_none() {
        return None;
    }

    let db = ctx
        .data::<Arc<Database>>()
        .expect("Missing database in the context");

    let query = WhereNftSchema {
        collection_id,
        nft_id,
        ..Default::default()
    };

    let filter = FilterNftSchema {
        where_: Some(query),
        ..Default::default()
    };

    db.nfts()
        .fetch_nfts(filter)
        .await
        .unwrap_or_default()
        .first()
        .cloned()
}

async fn fetch_total_collection_trait(
    ctx: &Context<'_>,
    collection_id: Option<String>,
) -> Option<i64> {
    if collection_id.is_none() {
        return None;
    }

    let collection_id = collection_id.as_ref().unwrap();

    let db = ctx
        .data::<Arc<Database>>()
        .expect("Missing database in the context");

    db.attributes()
        .total_collection_trait(collection_id)
        .await
        .ok()
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

async fn fetch_nft_top_offer(ctx: &Context<'_>, nft_id: &str) -> Option<String> {
    let db = ctx
        .data::<Arc<Database>>()
        .expect("Missing database in the context");

    let res = db.bids().fetch_nft_top_offer(nft_id).await;
    if res.is_err() {
        return None;
    }

    res.unwrap().map(|e| e.to_plain_string())
}

async fn fetch_total_nft(
    ctx: &Context<'_>,
    collection_id: Option<String>,
    wallet_address: Option<String>,
    default_amount: i64,
) -> Option<i64> {
    if collection_id.is_none() {
        return None;
    }

    if wallet_address.is_none() {
        return Some(default_amount);
    }

    let collection_id = collection_id.as_ref().unwrap();
    let wallet_address = wallet_address.as_ref().unwrap();

    let db = ctx
        .data::<Arc<Database>>()
        .expect("Missing database in the context");

    db.nfts()
        .fetch_total_nft(&wallet_address, collection_id)
        .await
        .ok()
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
