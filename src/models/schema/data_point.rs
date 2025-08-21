use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, SimpleObject, ToSchema)]
#[graphql(name = "DataPoint")]
pub struct DataPointSchema {
    pub x: Option<DateTime<Utc>>,
    pub y: Option<i64>,
}

pub fn validate_data_set(
    start_date: &DateTime<Utc>,
    end_date: &DateTime<Utc>,
    interval: &PgInterval,
) -> bool {
    let interval = (interval.days as i64) * 86400 * 1_000_000 + interval.microseconds;
    let diff = end_date.timestamp_micros() - start_date.timestamp_micros();
    let total_data = diff / interval;

    total_data <= 500
}
