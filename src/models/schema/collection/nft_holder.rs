use async_graphql::InputObject;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NftHolderSchema {
    pub address: Option<String>,
    pub quantity: Option<i64>,
    pub mint: Option<i64>,
    pub send: Option<i64>,
    pub receive: Option<i64>,
}

#[async_graphql::Object]
impl NftHolderSchema {
    async fn address(&self) -> Option<&str> {
        self.address.as_ref().map(|e| e.as_str())
    }

    async fn quantity(&self) -> Option<i64> {
        self.quantity
    }

    async fn mint(&self) -> Option<i64> {
        self.mint
    }

    async fn send(&self) -> Option<i64> {
        self.send
    }

    async fn receive(&self) -> Option<i64> {
        self.receive
    }
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
