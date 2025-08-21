use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use sqlx::postgres::types::PgInterval;

use crate::utils::string_utils;

pub fn deserialize_pg_interval<'de, D>(deserializer: D) -> Result<PgInterval, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    string_utils::str_to_pginterval(&s).map_err(serde::de::Error::custom)
}

pub fn deserialize_i64_to_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp = i64::deserialize(deserializer)?;
    DateTime::from_timestamp_millis(timestamp)
        .ok_or_else(|| serde::de::Error::custom("Invalid timestamp"))
}
