use async_graphql::{Enum, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(name = "CollectionTopWallet", rename_fields = "snake_case")]
pub struct TopWalletSchema {
    pub address: Option<String>,
    pub total: Option<i64>,
    pub volume: Option<BigDecimal>,
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
