use crate::models::marketplace::APT_DECIMAL;
use async_graphql::{Enum, InputObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TopWalletSchema {
    pub address: Option<String>,
    pub total: Option<i64>,
    pub volume: Option<BigDecimal>,
}

#[async_graphql::Object]
impl TopWalletSchema {
    async fn total(&self) -> Option<i64> {
        self.total
    }

    async fn address(&self) -> Option<&str> {
        self.address.as_ref().map(|e| e.as_str())
    }

    async fn volume(&self) -> Option<String> {
        self.volume
            .as_ref()
            .map(|e| (e / APT_DECIMAL).to_plain_string())
    }
}

#[derive(Clone, Debug, Default, InputObject, Deserialize)]
pub struct FilterTopWalletSchema {
    #[graphql(name = "where")]
    pub where_: WhereTopWalletSchema,
    pub limit: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereTopWalletSchema {
    #[graphql(name = "type")]
    pub type_: TopWalletType,
    pub collection_id: String,
    pub interval: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
#[graphql(rename_items = "snake_case")]
pub enum TopWalletType {
    Buyer,
    Seller,
}

impl Default for TopWalletType {
    fn default() -> Self {
        Self::Buyer
    }
}
