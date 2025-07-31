use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AttributeSchema {
    pub attr_type: String,
    pub values: serde_json::Value,
}

#[async_graphql::Object]
impl AttributeSchema {
    #[graphql(name = "type")]
    async fn attr_type(&self) -> &str {
        &self.attr_type
    }

    async fn values(&self) -> Vec<String> {
        serde_json::from_value(self.values.clone()).unwrap_or_default()
    }
}
