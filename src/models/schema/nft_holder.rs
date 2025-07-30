use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NftHolder {
    pub address: Option<String>,
    pub quantity: Option<i64>,
    pub mint: Option<i64>,
    pub send: Option<i64>,
    pub receive: Option<i64>,
}

#[async_graphql::Object]
impl NftHolder {
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
