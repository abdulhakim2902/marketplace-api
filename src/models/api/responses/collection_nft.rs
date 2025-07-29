use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CollectionNft {
    pub id: Option<String>,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub listing_price: Option<BigDecimal>,
    pub listing_usd_price: Option<BigDecimal>,
    pub last_sale: Option<BigDecimal>,
    pub listed_at: Option<DateTime<Utc>>,
    pub top_offer: Option<BigDecimal>,
    pub royalty: Option<BigDecimal>,
    pub rarity_score: Option<BigDecimal>,
}
