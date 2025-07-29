use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NftOffer {
    pub price: Option<BigDecimal>,
    pub usd_price: Option<BigDecimal>,
    pub from: Option<String>,
    pub expired_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
}
