use serde::{Deserialize, Deserializer};
use sqlx::postgres::types::PgInterval;
use validator::Validate;

use crate::{models::api::requests::PagingRequest, utils::string_utils};

#[derive(Deserialize, Validate, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilterCollection {
    #[serde(default, deserialize_with = "deserialize_pg_interval")]
    pub interval: Option<PgInterval>,
    #[serde(flatten)]
    pub paging: PagingRequest,
}

fn deserialize_pg_interval<'de, D>(deserializer: D) -> Result<Option<PgInterval>, D::Error>
where
    D: Deserializer<'de>,
{
    let o: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = o {
        string_utils::str_to_pginterval(&s).map_err(serde::de::Error::custom)
    } else {
        Ok(None)
    }
}
