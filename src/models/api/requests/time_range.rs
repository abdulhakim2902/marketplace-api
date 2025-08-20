use chrono::{DateTime, Utc};
use serde::Deserialize;
use utoipa::ToSchema;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::utils::{de_utils, string_utils::CustomInterval};

#[derive(Deserialize, Validate, Debug, Clone, utoipa::IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TimeRange {
    #[serde(deserialize_with = "de_utils::deserialize_i64_to_datetime")]
    #[schema(value_type = i64)]
    pub start_time: DateTime<Utc>,
    #[serde(deserialize_with = "de_utils::deserialize_i64_to_datetime")]
    #[schema(value_type = i64)]
    pub end_time: DateTime<Utc>,
    #[serde(deserialize_with = "de_utils::deserialize_pg_interval")]
    #[schema(value_type = String)]
    pub interval: CustomInterval,
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
