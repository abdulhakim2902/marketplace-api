pub mod attribute;
pub mod nft_change;
pub mod nft_distribution;
pub mod nft_holder;
pub mod profit_leaderboard;
pub mod top_buyer;
pub mod top_seller;
pub mod trending;

use async_graphql::{Context, InputObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::models::schema::{
    fetch_collection_past_floor, fetch_collection_sale, fetch_collection_top_offer,
    fetch_total_collection_offer, fetch_total_collection_trait, fetch_total_nft,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow)]
pub struct CollectionSchema {
    pub id: Option<String>,
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
    pub total_volume: Option<BigDecimal>,
    pub total_sale: Option<i64>,
    pub total_owner: Option<i64>,
    pub floor: Option<BigDecimal>,
    pub listed: Option<i64>,
}

#[async_graphql::Object]
impl CollectionSchema {
    async fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|e| e.as_str())
    }

    async fn slug(&self) -> Option<&str> {
        self.slug.as_ref().map(|e| e.as_str())
    }

    async fn supply(&self) -> Option<i64> {
        self.supply
    }

    async fn title(&self) -> Option<&str> {
        self.title.as_ref().map(|e| e.as_str())
    }

    async fn description(&self) -> Option<&str> {
        self.description.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "cover_url")]
    async fn cover_url(&self) -> Option<&str> {
        self.cover_url.as_ref().map(|e| e.as_str())
    }

    async fn floor(&self) -> Option<String> {
        self.floor.as_ref().map(|e| e.to_string())
    }

    async fn listed(&self) -> Option<i64> {
        self.listed
    }

    async fn verified(&self) -> Option<bool> {
        self.verified
    }

    async fn website(&self) -> Option<&str> {
        self.website.as_ref().map(|e| e.as_str())
    }

    async fn discord(&self) -> Option<&str> {
        self.discord.as_ref().map(|e| e.as_str())
    }

    async fn twitter(&self) -> Option<&str> {
        self.twitter.as_ref().map(|e| e.as_str())
    }

    async fn royalty(&self) -> Option<String> {
        self.royalty.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "total_volume")]
    async fn total_volume(&self) -> Option<String> {
        self.total_volume.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "total_sale")]
    async fn total_sale(&self) -> Option<i64> {
        self.total_sale
    }

    #[graphql(name = "total_owner")]
    async fn total_owner(&self) -> Option<i64> {
        self.total_owner
    }

    #[graphql(name = "total_nft")]
    async fn total_nft(&self, ctx: &Context<'_>, wallet_address: Option<String>) -> Option<i64> {
        fetch_total_nft(
            ctx,
            self.id.clone(),
            wallet_address,
            self.supply.unwrap_or_default(),
        )
        .await
    }

    async fn sale(
        &self,
        ctx: &Context<'_>,
        interval: Option<String>,
    ) -> Option<CollectionSaleSchema> {
        fetch_collection_sale(ctx, self.id.clone(), interval).await
    }

    #[graphql(name = "top_offer")]
    async fn top_offer(&self, ctx: &Context<'_>) -> Option<String> {
        fetch_collection_top_offer(ctx, self.id.clone()).await
    }

    #[graphql(name = "past_floor")]
    async fn past_floor(&self, ctx: &Context<'_>, interval: Option<String>) -> Option<String> {
        fetch_collection_past_floor(ctx, self.id.clone(), interval).await
    }

    #[graphql(name = "total_trait")]
    async fn total_trait(&self, ctx: &Context<'_>) -> Option<i64> {
        fetch_total_collection_trait(ctx, self.id.clone()).await
    }

    #[graphql(name = "total_offer")]
    async fn total_offer(&self, ctx: &Context<'_>) -> Option<String> {
        fetch_total_collection_offer(ctx, self.id.clone()).await
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow)]
pub struct CollectionSaleSchema {
    pub total: Option<i64>,
    pub volume: Option<BigDecimal>,
    pub volume_usd: Option<BigDecimal>,
}

#[async_graphql::Object]
impl CollectionSaleSchema {
    async fn total(&self) -> Option<i64> {
        self.total
    }

    async fn volume(&self) -> Option<String> {
        self.volume.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "volume_usd")]
    async fn volume_usd(&self) -> Option<String> {
        self.volume_usd.as_ref().map(|e| e.to_string())
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterCollectionSchema {
    #[graphql(name = "where")]
    pub where_: Option<WhereCollectionSchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereCollectionSchema {
    pub wallet_address: Option<String>,
    pub collection_id: Option<String>,
}
