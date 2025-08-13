use async_graphql::InputObject;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow)]
pub struct AttributeSchema {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub nft_id: Uuid,
    pub attr_type: String,
    pub value: String,
    pub rarity: Option<BigDecimal>,
    pub score: Option<BigDecimal>,
}

#[async_graphql::Object]
impl AttributeSchema {
    async fn id(&self) -> String {
        self.id.to_string()
    }

    #[graphql(name = "collection_id")]
    async fn collection_id(&self) -> String {
        self.collection_id.to_string()
    }

    #[graphql(name = "nft_id")]
    async fn nft_id(&self) -> String {
        self.nft_id.to_string()
    }

    #[graphql(name = "type")]
    async fn attr_type(&self) -> &str {
        &self.attr_type
    }

    async fn value(&self) -> &str {
        &self.value
    }

    async fn score(&self) -> Option<String> {
        self.score.as_ref().map(|e| e.to_plain_string())
    }

    async fn rarity(&self) -> Option<String> {
        self.rarity.as_ref().map(|e| e.to_plain_string())
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterAttributeSchema {
    #[graphql(name = "where")]
    pub where_: Option<WhereAttributeSchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereAttributeSchema {
    pub id: Option<String>,
    pub nft_id: Option<String>,
    pub collection_id: Option<String>,
}
