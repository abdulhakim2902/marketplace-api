use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NftActivity {
    pub tx_type: Option<String>,
    pub tx_index: Option<i64>,
    pub tx_id: Option<String>,
    pub seller: Option<String>,
    pub buyer: Option<String>,
    pub price: Option<i64>,
    pub usd_price: Option<BigDecimal>,
    pub market_name: Option<String>,
    pub market_contract_id: Option<String>,
    pub time: Option<DateTime<Utc>>,
}
