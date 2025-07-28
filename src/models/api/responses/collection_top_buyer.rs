use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionTopBuyer {
    pub buyer: Option<String>,
    pub bought: Option<i64>,
    pub volume: Option<BigDecimal>,
}
