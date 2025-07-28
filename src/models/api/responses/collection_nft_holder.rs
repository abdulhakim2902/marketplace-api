use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionNftHolder {
    pub address: Option<String>,
    pub quantity: Option<i64>,
    pub mint: Option<i64>,
    pub send: Option<i64>,
    pub receive: Option<i64>,
}
