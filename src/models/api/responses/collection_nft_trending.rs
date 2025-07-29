use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionNftTrending {
    pub nft_image_url: Option<String>,
    pub nft_name: Option<String>,
    pub tx_frequency: Option<i64>,
    pub last_price: Option<BigDecimal>,
}
