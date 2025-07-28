use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionNftChange {
    pub address: Option<String>,
    pub change: Option<i64>,
    pub quantity: Option<i64>,
}
