use async_graphql::{Context, Enum, InputObject};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::models::{
    marketplace::APT_DECIMAL,
    schema::{OrderingType, collection::CollectionSchema, fetch_collection, fetch_nft_top_offer},
};

#[derive(Clone, Debug, Deserialize, Serialize, FromRow)]
pub struct NftSchema {
    pub id: String,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub collection_id: Option<String>,
    pub burned: Option<bool>,
    pub properties: Option<serde_json::Value>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub animation_url: Option<String>,
    pub avatar_url: Option<String>,
    pub external_url: Option<String>,
    pub youtube_url: Option<String>,
    pub background_color: Option<String>,
    pub royalty: Option<BigDecimal>,
    pub version: Option<String>,
    pub rank: Option<i64>,
    pub score: Option<BigDecimal>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_sale: Option<i64>,
    pub listed_at: Option<DateTime<Utc>>,
    pub list_price: Option<i64>,
    pub list_usd_price: Option<BigDecimal>,
    pub attributes: Option<serde_json::Value>,
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
        self.last_sale
            .as_ref()
            .map(|e| (BigDecimal::from(*e) / APT_DECIMAL).to_plain_string())
    }

    #[graphql(name = "listed_at")]
    async fn listed_at(&self) -> Option<String> {
        self.listed_at.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "list_price")]
    async fn list_price(&self) -> Option<String> {
        self.list_price
            .as_ref()
            .map(|e| (BigDecimal::from(*e) / APT_DECIMAL).to_plain_string())
    }

    #[graphql(name = "list_usd_price")]
    async fn list_usd_price(&self) -> Option<String> {
        self.list_usd_price.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "rarity_score")]
    async fn score(&self) -> Option<String> {
        self.score.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "rank")]
    async fn rank(&self) -> Option<i64> {
        self.rank
    }

    #[graphql(name = "animation_url")]
    async fn animation_url(&self) -> Option<&str> {
        self.animation_url.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "avatar_url")]
    async fn avatar_url(&self) -> Option<&str> {
        self.avatar_url.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "youtube_url")]
    async fn youtube_url(&self) -> Option<&str> {
        self.youtube_url.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "external_url")]
    async fn external_url(&self) -> Option<&str> {
        self.external_url.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "background_color")]
    async fn background_color(&self) -> Option<&str> {
        self.background_color.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "attributes")]
    async fn attributes(&self) -> Vec<NftAttributeSchema> {
        self.attributes
            .clone()
            .map(|e| serde_json::from_value::<Vec<NftAttributeSchema>>(e).unwrap_or_default())
            .unwrap_or_default()
    }

    #[graphql(name = "top_offer")]
    async fn top_offer(&self, ctx: &Context<'_>) -> Option<String> {
        fetch_nft_top_offer(ctx, &self.id).await
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        fetch_collection(ctx, self.collection_id.clone()).await
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, InputObject)]
pub struct NftAttributeSchema {
    pub attr_type: Option<String>,
    pub value: Option<String>,
    pub rarity: Option<BigDecimal>,
    pub score: Option<BigDecimal>,
}

#[async_graphql::Object()]
impl NftAttributeSchema {
    #[graphql(name = "type")]
    async fn attr_type(&self) -> Option<&str> {
        self.attr_type.as_ref().map(|e| e.as_str())
    }

    async fn value(&self) -> Option<&str> {
        self.value.as_ref().map(|e| e.as_str())
    }

    async fn rarity(&self) -> Option<String> {
        self.rarity.as_ref().map(|e| e.to_string())
    }

    async fn score(&self) -> Option<String> {
        self.score.as_ref().map(|e| e.to_string())
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterNftSchema {
    #[graphql(name = "where")]
    pub where_: Option<WhereNftSchema>,
    #[graphql(name = "order_by")]
    pub order_by: Option<OrderNftSchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereNftSchema {
    #[graphql(name = "type")]
    pub type_: Option<FilterType>,
    pub search: Option<String>,
    pub wallet_address: Option<String>,
    pub collection_id: Option<String>,
    pub nft_id: Option<String>,
    pub burned: Option<bool>,
    pub rarity: Option<WhereNftRankSchema>,
    pub market_contract_ids: Option<Vec<String>>,
    pub price: Option<WhereNftPriceSchema>,
    pub attributes: Option<Vec<WhereNftAttributeSchema>>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereNftAttributeSchema {
    #[graphql(name = "type")]
    pub type_: String,
    pub values: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereNftRankSchema {
    pub min: i64,
    pub max: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereNftPriceSchema {
    #[graphql(name = "type")]
    pub type_: CoinType,
    pub range: WhereNftPriceRangeSchema,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereNftPriceRangeSchema {
    pub min: BigDecimal,
    pub max: Option<BigDecimal>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct OrderNftSchema {
    pub price: Option<OrderingType>,
    pub rarity: Option<OrderingType>,
    pub listed_at: Option<OrderingType>,
    pub received_at: Option<OrderingType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
#[graphql(rename_items = "snake_case")]
pub enum CoinType {
    APT,
    USD,
}

impl Default for CoinType {
    fn default() -> Self {
        Self::APT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
#[graphql(rename_items = "snake_case")]
pub enum FilterType {
    All,
    Listed,
    HasOffer,
}

impl Default for FilterType {
    fn default() -> Self {
        Self::All
    }
}
