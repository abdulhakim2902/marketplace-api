use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Attribute {
    pub collection_id: Option<String>,
    pub nft_id: Option<String>,
    pub attr_type: Option<String>,
    pub value: Option<String>,
    pub rarity: Option<BigDecimal>,
    pub score: Option<BigDecimal>,
}
