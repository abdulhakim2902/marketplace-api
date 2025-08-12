use crate::database::token_prices::ITokenPrices;
use crate::models::schema::collection::FilterCollectionSchema;
use crate::models::schema::nft::FilterNftSchema;
use crate::{
    database::{
        Database, IDatabase, attributes::IAttributes, bids::IBids, collections::ICollections,
        nfts::INfts,
    },
    models::schema::{
        collection::{CollectionSchema, WhereCollectionSchema},
        nft::{NftSchema, WhereNftSchema},
    },
};
use async_graphql::{Context, Enum};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::{Display, EnumString};

pub mod activity;
pub mod attribute;
pub mod collection;
pub mod data_point;
pub mod listing;
pub mod marketplace;
pub mod nft;
pub mod offer;
pub mod wallet;

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

async fn fetch_collection(
    ctx: &Context<'_>,
    collection_id: Option<String>,
) -> Option<CollectionSchema> {
    if collection_id.is_none() {
        return None;
    }

    let db = ctx
        .data::<Arc<Database>>()
        .expect("Missing database in the context");

    let query = WhereCollectionSchema {
        collection_id,
        ..Default::default()
    };

    let filter = FilterCollectionSchema {
        where_: Some(query),
        ..Default::default()
    };

    db.collections()
        .fetch_collections(filter)
        .await
        .unwrap_or_default()
        .first()
        .cloned()
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
