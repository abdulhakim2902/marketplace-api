use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionOffer {
    pub total_volume: Option<BigDecimal>,
    pub offer_volume: Option<BigDecimal>,
    pub bidder: Option<String>,
    pub total_offer: Option<i64>,
    pub last_update: DateTime<Utc>,
}
