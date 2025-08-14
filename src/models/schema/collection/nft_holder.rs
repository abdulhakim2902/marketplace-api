use async_graphql::{InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct NftHolderSchema {
    pub address: Option<String>,
    pub quantity: Option<i64>,
    pub mint: Option<i64>,
    pub send: Option<i64>,
    pub receive: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterNftHolderSchema {
    #[graphql(name = "where")]
    pub where_: WhereNftHolderSchema,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereNftHolderSchema {
    pub collection_id: String,
}
