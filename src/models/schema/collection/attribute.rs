use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(name = "CollectionAttribute", rename_fields = "snake_case")]
pub struct CollectionAttributeSchema {
    #[graphql(name = "type")]
    pub attr_type: String,
    pub values: serde_json::Value,
}
