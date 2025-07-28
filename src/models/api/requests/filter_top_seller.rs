use serde::Deserialize;
use sqlx::postgres::types::PgInterval;
use validator::Validate;

use crate::models::api::requests::deserialize_option_pg_interval;

#[derive(Deserialize, Validate, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilterTopSeller {
    #[serde(default, deserialize_with = "deserialize_option_pg_interval")]
    pub interval: Option<PgInterval>,
}
