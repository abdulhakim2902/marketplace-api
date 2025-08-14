use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct MarketplaceSchema {
    pub id: Uuid,
    pub name: String,
    pub contract_address: String,
}
