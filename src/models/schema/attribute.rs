use async_graphql::{InputObject, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::models::schema::{OperatorSchema, OrderingType, nft::QueryNftSchema};

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

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct QueryAttributeSchema {
    #[graphql(name = "_or")]
    pub _or: Option<Box<QueryAttributeSchema>>,
    #[graphql(name = "_and")]
    pub _and: Option<Box<QueryAttributeSchema>>,
    #[graphql(name = "_not")]
    pub _not: Option<Box<QueryAttributeSchema>>,
    pub id: Option<OperatorSchema<Uuid>>,
    pub collection_id: Option<OperatorSchema<Uuid>>,
    pub nft_id: Option<OperatorSchema<Uuid>>,
    #[graphql(name = "type")]
    pub attr_type: Option<OperatorSchema<String>>,
    pub value: Option<OperatorSchema<String>>,
    pub rarity: Option<OperatorSchema<BigDecimal>>,
    pub score: Option<OperatorSchema<BigDecimal>>,
    pub nft: Option<QueryNftSchema>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct OrderAttributeSchema {
    pub id: Option<OrderingType>,
    pub collection_id: Option<OrderingType>,
    pub nft_id: Option<OrderingType>,
    #[graphql(name = "type")]
    pub attr_type: Option<OrderingType>,
    pub value: Option<OrderingType>,
    pub rarity: Option<OrderingType>,
    pub score: Option<OrderingType>,
}
