use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Collection {
    pub id: Option<String>,
    pub slug: Option<String>,
    pub supply: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub floor: Option<BigDecimal>,
    pub prev_floor: Option<BigDecimal>,
    pub owners: Option<i64>,
    pub sales: Option<i64>,
    pub listed: Option<i64>,
    pub top_offer: Option<BigDecimal>,
    pub volume: Option<BigDecimal>,
    pub average: Option<BigDecimal>,
    pub volume_usd: Option<BigDecimal>,
}
