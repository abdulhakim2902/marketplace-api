use std::sync::Arc;

use async_graphql::Context;

use crate::{
    database::{
        Database, IDatabase, activities::IActivities, attributes::IAttributes, bids::IBids,
        collections::ICollections, nfts::INfts,
    },
    models::schema::{
        collection::{CollectionSaleSchema, CollectionSchema},
        nft::NftSchema,
    },
    utils::string_utils,
};

pub mod activity;
pub mod bid;
pub mod collection;
pub mod collection_trending;
pub mod data_point;
pub mod listing;
pub mod nft;
pub mod nft_change;
pub mod nft_distribution;
pub mod nft_holder;
pub mod profit_leaderboard;
pub mod top_buyer;
pub mod top_seller;

async fn fetch_collection(
    ctx: &Context<'_>,
    collection_id: Option<String>,
) -> Option<CollectionSchema> {
    let db = ctx
        .data::<Arc<Database>>()
        .expect("Missing database in the context");

    db.collections()
        .fetch_collections(collection_id, None, 1, 0)
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
    let db = ctx
        .data::<Arc<Database>>()
        .expect("Missing database in the context");

    db.nfts()
        .fetch_nfts(nft_id, collection_id, None, 1, 0)
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

async fn fetch_collection_past_floor(
    ctx: &Context<'_>,
    collection_id: Option<String>,
    interval: Option<String>,
) -> Option<String> {
    if collection_id.is_none() {
        return None;
    }

    let i =
        string_utils::str_to_pginterval(&interval.unwrap_or_default()).expect("Invalid interval");

    let collection_id = collection_id.as_ref().unwrap();
    let db = ctx
        .data::<Arc<Database>>()
        .expect("Missing database in the context");

    let res = db.activities().fetch_past_floor(collection_id, i).await;
    if res.is_err() {
        None
    } else {
        res.unwrap().map(|e| e.to_plain_string())
    }
}

async fn fetch_collection_top_offer(
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

    let res = db.bids().fetch_collection_top_offer(collection_id).await;
    if res.is_err() {
        return None;
    }

    res.unwrap().map(|e| e.to_plain_string())
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

async fn fetch_collection_sale(
    ctx: &Context<'_>,
    collection_id: Option<String>,
    interval: Option<String>,
) -> Option<CollectionSaleSchema> {
    if collection_id.is_none() {
        return None;
    }

    let i =
        string_utils::str_to_pginterval(&interval.unwrap_or_default()).expect("Invalid interval");

    let collection_id = collection_id.as_ref().unwrap();
    let db = ctx
        .data::<Arc<Database>>()
        .expect("Missing database in the context");

    db.activities().fetch_sale(collection_id, i).await.ok()
}

async fn fetch_nft_rarity_score(
    ctx: &Context<'_>,
    nft_id: &str,
    collection_id: Option<String>,
) -> Option<String> {
    if collection_id.is_none() {
        return None;
    }

    let collection_id = collection_id.as_ref().unwrap();
    let db = ctx
        .data::<Arc<Database>>()
        .expect("Missing database in the context");

    let res = db
        .attributes()
        .nft_rarity_scores(collection_id, nft_id)
        .await;
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
