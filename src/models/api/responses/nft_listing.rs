use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NftListing {
    pub price: Option<i64>,
    pub usd_price: Option<BigDecimal>,
    pub market_name: Option<String>,
    pub market_contract_id: Option<String>,
    pub from: Option<String>,
}
