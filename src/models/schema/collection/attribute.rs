use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionAttributeSchema {
    pub type_: String,
    pub values: serde_json::Value,
}

#[async_graphql::Object]
impl CollectionAttributeSchema {
    #[graphql(name = "type")]
    async fn attr_type(&self) -> &str {
        &self.type_
    }

    async fn values(&self) -> Vec<String> {
        serde_json::from_value(self.values.clone()).unwrap_or_default()
    }
}
