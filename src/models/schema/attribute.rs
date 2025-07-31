use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AttributeSchema {
    pub collection_id: Option<String>,
    pub nft_id: Option<String>,
    pub attr_type: Option<String>,
    pub value: Option<String>,
    pub rarity: Option<BigDecimal>,
    pub score: Option<BigDecimal>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionAttributeSchema {
    pub attr_type: String,
    pub values: serde_json::Value,
}

#[async_graphql::Object]
impl CollectionAttributeSchema {
    #[graphql(name = "type")]
    async fn attr_type(&self) -> &str {
        &self.attr_type
    }

    async fn values(&self) -> Vec<String> {
        serde_json::from_value(self.values.clone()).unwrap_or_default()
    }
}
