use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(name = "CollectionNftChange", rename_fields = "snake_case")]
pub struct NftChangeSchema {
    pub address: Option<String>,
    pub change: Option<i64>,
    pub quantity: Option<i64>,
}
