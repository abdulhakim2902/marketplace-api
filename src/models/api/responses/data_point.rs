use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DataPoint {
    pub x: Option<DateTime<Utc>>,
    pub y: Option<i64>,
}
