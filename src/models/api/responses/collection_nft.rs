use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionNft {
    pub id: Option<String>,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub listing_price: Option<i64>,
    pub last_sale: Option<i64>,
    pub listed_at: Option<DateTime<Utc>>,
    pub top_offer: Option<i64>,
    pub royalty: Option<BigDecimal>,
}
