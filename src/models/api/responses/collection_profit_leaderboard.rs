use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionProfitLeaderboard {
    pub address: Option<String>,
    pub spent: Option<BigDecimal>,
    pub bought: Option<i64>,
    pub sold: Option<i64>,
    pub total_profit: Option<BigDecimal>,
    pub profit_percentage: Option<BigDecimal>,
}
