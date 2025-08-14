use async_graphql::{InputObject, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct AttributeSchema {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub nft_id: Uuid,
    #[graphql(name = "type")]
    pub attr_type: String,
    pub value: String,
    pub rarity: Option<BigDecimal>,
    pub score: Option<BigDecimal>,
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
