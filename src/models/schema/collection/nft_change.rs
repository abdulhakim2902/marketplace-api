use async_graphql::{InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct NftChangeSchema {
    pub address: Option<String>,
    pub change: Option<i64>,
    pub quantity: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterNftChangeSchema {
    #[graphql(name = "where")]
    pub where_: WhereNftChangeSchema,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereNftChangeSchema {
    pub collection_id: String,
    pub interval: Option<String>,
}
