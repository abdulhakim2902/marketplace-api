use crate::models::schema::{
    OperatorSchema, OrderingType, collection::CollectionSchema, fetch_collection,
    fetch_nft_top_offer,
};
use async_graphql::{ComplexObject, Context, Enum, InputObject, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct NftSchema {
    pub id: Uuid,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub collection_id: Option<Uuid>,
    pub burned: Option<bool>,
    pub properties: Option<serde_json::Value>,
    pub description: Option<String>,
    #[graphql(name = "media_url")]
    pub image_url: Option<String>,
    pub token_id: Option<String>,
    pub animation_url: Option<String>,
    pub avatar_url: Option<String>,
    pub external_url: Option<String>,
    pub youtube_url: Option<String>,
    pub background_color: Option<String>,
    pub royalty: Option<BigDecimal>,
    pub version: Option<String>,
    pub ranking: Option<i64>,
    pub rarity: Option<BigDecimal>,
}

#[ComplexObject]
impl NftSchema {
    #[graphql(name = "top_offer")]
    async fn top_offer(&self, ctx: &Context<'_>) -> Option<String> {
        fetch_nft_top_offer(ctx, &self.id.to_string()).await
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        fetch_collection(ctx, self.collection_id.as_ref().map(|e| e.to_string())).await
    }

    // TODO: FETCH ATTRIBUTES
    // TODO: FETCH LISTINGS
    // TODO: FETCH BIDS
    // TODO: FETCH ACTIONS
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct QueryNftSchema {
    #[graphql(name = "_or")]
    pub _or: Option<Box<QueryNftSchema>>,
    #[graphql(name = "_and")]
    pub _and: Option<Box<QueryNftSchema>>,
    #[graphql(name = "_not")]
    pub _not: Option<Box<QueryNftSchema>>,
    pub id: Option<OperatorSchema<Uuid>>,
    pub name: Option<OperatorSchema<String>>,
    pub owner: Option<OperatorSchema<String>>,
    pub collection_id: Option<OperatorSchema<Uuid>>,
    pub burned: Option<OperatorSchema<bool>>,
    // pub properties: Option<serde_json::Value>,
    pub description: Option<OperatorSchema<String>>,
    #[graphql(name = "media_url")]
    pub image_url: Option<OperatorSchema<String>>,
    pub token_id: Option<OperatorSchema<String>>,
    pub animation_url: Option<OperatorSchema<String>>,
    pub avatar_url: Option<OperatorSchema<String>>,
    pub external_url: Option<OperatorSchema<String>>,
    pub youtube_url: Option<OperatorSchema<String>>,
    pub background_color: Option<OperatorSchema<String>>,
    pub royalty: Option<OperatorSchema<BigDecimal>>,
    pub version: Option<OperatorSchema<String>>,
    pub ranking: Option<OperatorSchema<i64>>,
    pub rarity: Option<OperatorSchema<BigDecimal>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct OrderNftSchemas {
    pub id: Option<OrderingType>,
    pub name: Option<OrderingType>,
    pub owner: Option<OrderingType>,
    pub collection_id: Option<OrderingType>,
    pub burned: Option<OrderingType>,
    // pub properties: Option<serde_json::Value>,
    pub description: Option<OrderingType>,
    #[graphql(name = "media_url")]
    pub image_url: Option<OrderingType>,
    pub token_id: Option<OrderingType>,
    pub animation_url: Option<OrderingType>,
    pub avatar_url: Option<OrderingType>,
    pub external_url: Option<OrderingType>,
    pub youtube_url: Option<OrderingType>,
    pub background_color: Option<OrderingType>,
    pub royalty: Option<OrderingType>,
    pub version: Option<OrderingType>,
    pub ranking: Option<OrderingType>,
    pub rarity: Option<OrderingType>,
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
