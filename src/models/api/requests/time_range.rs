use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use sqlx::postgres::types::PgInterval;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::utils::de_utils;

#[derive(Deserialize, Validate, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimeRange {
    #[serde(
        deserialize_with = "de_utils::deserialize_i64_to_datetime",
        default = "default_time"
    )]
    pub start_time: DateTime<Utc>,
    #[serde(
        deserialize_with = "de_utils::deserialize_i64_to_datetime",
        default = "default_time"
    )]
    pub end_time: DateTime<Utc>,
    #[serde(
        deserialize_with = "de_utils::deserialize_pg_interval",
        default = "default_interval"
    )]
    pub interval: PgInterval,
    pub api_key_id: Option<String>,
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

fn default_time() -> DateTime<Utc> {
    Utc::now()
}

fn default_interval() -> PgInterval {
    let duration = Duration::hours(1);
    PgInterval {
        months: 0,
        days: duration.num_days() as i32,
        microseconds: (duration.num_seconds() % 86400) * 1_000_000,
    }
}
