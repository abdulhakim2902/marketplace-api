use std::sync::Arc;

use async_graphql::Context;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::{Database, IDatabase, attributes::IAttributes};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Nft {
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
}

#[async_graphql::Object]
impl Nft {
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

    async fn rarity_score(&self, ctx: &Context<'_>) -> Option<String> {
        if self.collection_id.is_none() {
            return None;
        }

        let collection_id = self.collection_id.as_ref().unwrap();
        let nft_id = self.id.as_str();

        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing service in the context");

        let res = db
            .attributes()
            .nft_rarity_scores(collection_id, nft_id)
            .await;

        if res.is_err() {
            return None;
        }

        res.unwrap().map(|e| e.to_plain_string())
    }
}
