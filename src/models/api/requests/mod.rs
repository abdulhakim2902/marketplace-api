use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use serde_with::{DisplayFromStr, PickFirst, serde_as};
use sqlx::postgres::types::PgInterval;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::utils::string_utils;

pub mod filter_offer;

#[serde_as]
#[derive(Deserialize, Debug, Clone, Validate, Default)]
#[serde(rename_all = "camelCase")]
pub struct PagingRequest {
    #[validate(range(min = 1, message = "page must be greater than 0"))]
    #[serde_as(as = "PickFirst<(_, DisplayFromStr)>")]
    #[serde(default = "default_page")]
    pub page: i64,

    #[validate(range(min = 1, message = "page_size must be greater than 0"))]
    #[serde_as(as = "PickFirst<(_, DisplayFromStr)>")]
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

#[derive(Deserialize, Validate, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimeRange {
    #[serde(deserialize_with = "deserialize_i64_to_datetime")]
    pub start_time: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_i64_to_datetime")]
    pub end_time: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_pg_interval")]
    pub interval: PgInterval,
}

impl TimeRange {
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let interval = (self.interval.days as i64) * 86400 * 1_000_000 + self.interval.microseconds;
        let diff = self.end_time.timestamp_micros() - self.start_time.timestamp_micros();
        let total_data = diff / interval;

        if total_data > 400 {
            let errors = &mut ValidationErrors::new();
            errors.add("interval", ValidationError::new("Dataset to large"));
            Err(errors.clone())
        } else {
            Ok(())
        }
    }
}

pub fn deserialize_i64_to_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp = i64::deserialize(deserializer)?;
    DateTime::from_timestamp_millis(timestamp)
        .ok_or_else(|| serde::de::Error::custom("Invalid timestamp"))
}

fn deserialize_pg_interval<'de, D>(deserializer: D) -> Result<PgInterval, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let res = string_utils::str_to_pginterval(&s).map_err(serde::de::Error::custom)?;

    res.ok_or(anyhow!("Invalid interval format"))
        .map_err(serde::de::Error::custom)
}

pub fn default_limit() -> i64 {
    10
}

pub fn default_offset() -> i64 {
    0
}
