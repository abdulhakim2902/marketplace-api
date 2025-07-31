use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TopSellerSchema {
    pub seller: Option<String>,
    pub sold: Option<i64>,
    pub volume: Option<BigDecimal>,
}

#[async_graphql::Object]
impl TopSellerSchema {
    async fn sold(&self) -> Option<i64> {
        self.sold
    }

    async fn seller(&self) -> Option<&str> {
        self.seller.as_ref().map(|e| e.as_str())
    }

    async fn volume(&self) -> Option<String> {
        self.volume.as_ref().map(|e| e.to_string())
    }
}
