use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionInfo {
    pub id: Option<String>,
    pub slug: Option<String>,
    pub supply: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub verified: Option<bool>,
    pub website: Option<String>,
    pub discord: Option<String>,
    pub twitter: Option<String>,
    pub cover_url: Option<String>,
    pub royalty: Option<BigDecimal>,
    pub floor: Option<BigDecimal>,
    pub prev_floor: Option<BigDecimal>,
    pub owners: Option<i64>,
    pub sales: Option<i64>,
    pub listed: Option<i64>,
    pub top_offer: Option<BigDecimal>,
    pub average: Option<BigDecimal>,
    pub all_volume: Option<BigDecimal>,
    pub all_volume_usd: Option<BigDecimal>,
    pub sales_24h: Option<i64>,
    pub volume_24h: Option<BigDecimal>,
    pub volume_24h_usd: Option<BigDecimal>,
    pub total_offer: Option<BigDecimal>,
}
