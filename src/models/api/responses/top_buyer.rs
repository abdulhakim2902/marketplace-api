use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TopBuyer {
    pub buyer: Option<String>,
    pub bought: Option<i64>,
    pub volume: Option<BigDecimal>,
}

#[async_graphql::Object]
impl TopBuyer {
    async fn bought(&self) -> Option<i64> {
        self.bought
    }

    async fn buyer(&self) -> Option<&str> {
        self.buyer.as_ref().map(|e| e.as_str())
    }

    async fn volume(&self) -> Option<String> {
        self.volume.as_ref().map(|e| e.to_string())
    }
}
