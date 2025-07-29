use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NftInfo {
    pub id: Option<String>,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub list_price: Option<BigDecimal>,
    pub top_offer: Option<BigDecimal>,
}
