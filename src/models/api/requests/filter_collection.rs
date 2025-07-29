use serde::Deserialize;
use sqlx::postgres::types::PgInterval;
use validator::Validate;

use crate::models::api::requests::{default_limit, default_offset, deserialize_option_pg_interval};

#[derive(Deserialize, Validate, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilterCollection {
    #[serde(default, deserialize_with = "deserialize_option_pg_interval")]
    pub interval: Option<PgInterval>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub offset: i64,
}
