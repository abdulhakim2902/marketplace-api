pub mod attribute;
pub mod nft_change;
pub mod nft_distribution;
pub mod nft_holder;
pub mod profit_leaderboard;
pub mod stat;
pub mod top_wallet;
pub mod trending;

use crate::models::schema::{
    OrderingType, fetch_total_collection_offer, fetch_total_collection_trait, fetch_total_nft,
};
use async_graphql::{ComplexObject, Context, Enum, InputObject, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct CollectionSchema {
    pub id: Uuid,
    pub slug: Option<String>,
    pub supply: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub verified: Option<bool>,
    pub website: Option<String>,
    pub discord: Option<String>,
    pub twitter: Option<String>,
    pub royalty: Option<BigDecimal>,
    pub floor: Option<i64>,
    pub owners: Option<i64>,
    pub volume: Option<i64>,
    pub volume_usd: Option<BigDecimal>,
    pub sales: Option<i64>,
    pub listed: Option<i64>,
}

#[ComplexObject]
impl CollectionSchema {
    #[graphql(name = "market_cap")]
    async fn market_cap(&self) -> Option<i64> {
        self.supply
            .zip(self.floor)
            .map(|(supply, floor)| supply * floor)
    }

    #[graphql(name = "total_nft")]
    async fn total_nft(&self, ctx: &Context<'_>, wallet_address: Option<String>) -> Option<i64> {
        fetch_total_nft(
            ctx,
            Some(self.id.to_string()),
            wallet_address,
            self.supply.unwrap_or_default(),
        )
        .await
    }

    #[graphql(name = "total_trait")]
    async fn total_trait(&self, ctx: &Context<'_>) -> Option<i64> {
        fetch_total_collection_trait(ctx, Some(self.id.to_string())).await
    }

    #[graphql(name = "total_offer")]
    async fn total_offer(&self, ctx: &Context<'_>) -> Option<String> {
        fetch_total_collection_offer(ctx, Some(self.id.to_string())).await
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterCollectionSchema {
    #[graphql(name = "where")]
    pub where_: Option<WhereCollectionSchema>,
    #[graphql(name = "order_by")]
    pub order_by: Option<OrderCollectionSchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereCollectionSchema {
    pub search: Option<String>,
    pub wallet_address: Option<String>,
    pub collection_id: Option<String>,
    pub periods: Option<PeriodType>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct OrderCollectionSchema {
    pub volume: Option<OrderingType>,
    pub floor: Option<OrderingType>,
    pub owners: Option<OrderingType>,
    pub market_cap: Option<OrderingType>,
    pub sales: Option<OrderingType>,
    pub listed: Option<OrderingType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
#[graphql(rename_items = "snake_case")]
pub enum PeriodType {
    Hours1,
    Hours6,
    Days1,
    Days7,
}
