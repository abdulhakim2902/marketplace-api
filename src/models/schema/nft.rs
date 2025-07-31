use async_graphql::{Context, InputObject};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::schema::{
    collection::CollectionSchema, fetch_collection, fetch_nft_rarity_score, fetch_nft_top_offer,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NftSchema {
    pub id: String,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub collection_id: Option<String>,
    pub burned: Option<bool>,
    pub properties: Option<serde_json::Value>,
    pub description: Option<String>,
    pub background_color: Option<String>,
    pub image_data: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    pub avatar_url: Option<String>,
    pub external_url: Option<String>,
    pub uri: Option<String>,
    pub image_url: Option<String>,
    pub royalty: Option<BigDecimal>,
    pub version: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_sale: Option<BigDecimal>,
    pub listed_at: Option<DateTime<Utc>>,
    pub list_price: Option<BigDecimal>,
    pub list_usd_price: Option<BigDecimal>,
}

#[async_graphql::Object]
impl NftSchema {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|e| e.as_str())
    }

    async fn owner(&self) -> Option<&str> {
        self.owner.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "collection_id")]
    async fn collection_id(&self) -> Option<&str> {
        self.collection_id.as_ref().map(|e| e.as_str())
    }

    async fn burned(&self) -> Option<bool> {
        self.burned
    }

    async fn description(&self) -> Option<&str> {
        self.description.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "background_color")]
    async fn background_color(&self) -> Option<&str> {
        self.background_color.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "image_data")]
    async fn image_data(&self) -> Option<&str> {
        self.image_data.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "animation_url")]
    async fn animation_url(&self) -> Option<&str> {
        self.animation_url.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "youtube_url")]
    async fn youtube_url(&self) -> Option<&str> {
        self.youtube_url.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "avatar_url")]
    async fn avatar_url(&self) -> Option<&str> {
        self.avatar_url.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "external_url")]
    async fn external_url(&self) -> Option<&str> {
        self.external_url.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "uri")]
    async fn uri(&self) -> Option<&str> {
        self.uri.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "image_url")]
    async fn image_url(&self) -> Option<&str> {
        self.image_url.as_ref().map(|e| e.as_str())
    }

    async fn royalty(&self) -> Option<String> {
        self.royalty.as_ref().map(|e| e.to_string())
    }

    async fn version(&self) -> Option<&str> {
        self.version.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "updated_at")]
    async fn updated_at(&self) -> Option<String> {
        self.updated_at.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "last_sale")]
    async fn last_sale(&self) -> Option<String> {
        self.last_sale.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "listed_at")]
    async fn listed_at(&self) -> Option<String> {
        self.listed_at.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "list_price")]
    async fn list_price(&self) -> Option<String> {
        self.list_price.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "list_usd_price")]
    async fn list_usd_price(&self) -> Option<String> {
        self.list_usd_price.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "rarity_score")]
    async fn rarity_score(&self, ctx: &Context<'_>) -> Option<String> {
        fetch_nft_rarity_score(ctx, &self.id, self.collection_id.clone()).await
    }

    #[graphql(name = "top_offer")]
    async fn top_offer(&self, ctx: &Context<'_>) -> Option<String> {
        fetch_nft_top_offer(ctx, &self.id).await
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        fetch_collection(ctx, self.collection_id.clone()).await
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterNftSchema {
    #[graphql(name = "where")]
    pub where_: Option<WhereNftSchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereNftSchema {
    pub wallet_address: Option<String>,
    pub collection_id: Option<String>,
    pub nft_id: Option<String>,
}
