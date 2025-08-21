use async_graphql::{Enum, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use strum::{Display, EnumString};

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(name = "CollectionHolder", rename_fields = "snake_case")]
pub struct CollectionHolderSchema {
    pub collection_id: Uuid,
    pub current_holdings: Option<i64>,
    pub owner: Option<String>,
    pub sold: Option<i64>,
    pub sold_volume: Option<BigDecimal>,
    pub total_holdings: Option<i64>,
    pub total_holding_time: Option<BigDecimal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[graphql(rename_items = "snake_case")]
pub enum OrderHolderType {
    CurrentHoldings,
    Sold,
    AverageHold,
    AverageSold,
}

impl Default for OrderHolderType {
    fn default() -> Self {
        Self::CurrentHoldings
    }
}
