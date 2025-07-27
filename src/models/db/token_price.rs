use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TokenPrice {
    pub token_address: String,
    pub price: BigDecimal,
    pub created_at: DateTime<Utc>,
}
