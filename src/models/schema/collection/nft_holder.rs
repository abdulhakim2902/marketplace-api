use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(name = "CollectionNftHolder", rename_fields = "snake_case")]
pub struct NftHolderSchema {
    pub address: Option<String>,
    pub quantity: Option<i64>,
    pub mint: Option<i64>,
    pub send: Option<i64>,
    pub receive: Option<i64>,
}
