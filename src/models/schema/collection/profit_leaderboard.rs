use async_graphql::SimpleObject;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct ProfitLeaderboardSchema {
    pub address: Option<String>,
    pub spent: Option<BigDecimal>,
    pub bought: Option<i64>,
    pub sold: Option<i64>,
    pub total_profit: Option<BigDecimal>,
}
