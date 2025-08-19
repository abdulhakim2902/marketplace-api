use async_graphql::{ComplexObject, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(complex, name = "CollectionAttribute", rename_fields = "snake_case")]
pub struct CollectionAttributeSchema {
    #[graphql(name = "type")]
    pub attr_type: String,
    #[graphql(visible = false)]
    pub json: serde_json::Value,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(name = "CollectionAttributeValue", rename_fields = "snake_case")]
pub struct CollectionAttributeValueSchema {
    pub value: String,
    pub rarity: BigDecimal,
    pub score: BigDecimal,
}

#[ComplexObject]
impl CollectionAttributeSchema {
    async fn values(&self) -> Vec<CollectionAttributeValueSchema> {
        serde_json::from_value(self.json.clone()).unwrap_or_default()
    }
}
