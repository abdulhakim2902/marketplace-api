use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NftChangeSchema {
    pub address: Option<String>,
    pub change: Option<i64>,
    pub quantity: Option<i64>,
}

#[async_graphql::Object]
impl NftChangeSchema {
    async fn address(&self) -> Option<&str> {
        self.address.as_ref().map(|e| e.as_str())
    }

    async fn change(&self) -> Option<i64> {
        self.change
    }

    async fn quantity(&self) -> Option<i64> {
        self.quantity
    }
}
