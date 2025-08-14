use async_graphql::{Enum, InputObject, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct TopWalletSchema {
    pub address: Option<String>,
    pub total: Option<i64>,
    pub volume: Option<BigDecimal>,
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
