use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionTopSeller {
    pub seller: Option<String>,
    pub sold: Option<i64>,
    pub volume: Option<BigDecimal>,
}
