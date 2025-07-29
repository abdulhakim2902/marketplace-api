use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionActivity {
    pub tx_type: Option<String>,
    pub tx_index: Option<i64>,
    pub tx_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub price: Option<BigDecimal>,
    pub usd_price: Option<BigDecimal>,
    pub market_name: Option<String>,
    pub market_contract_id: Option<String>,
    pub time: Option<DateTime<Utc>>,
    pub nft_id: Option<String>,
    pub nft_name: Option<String>,
    pub nft_description: Option<String>,
    pub nft_image_url: Option<String>,
    pub quantity: Option<i64>,
}
