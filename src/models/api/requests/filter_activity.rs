use serde::Deserialize;
use validator::Validate;

use crate::models::api::requests::{default_limit, default_offset};

#[derive(Deserialize, Validate, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilterActivity {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub offset: i64,
}
