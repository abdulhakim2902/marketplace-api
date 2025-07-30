use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};

pub fn deserialize_i64_to_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp = i64::deserialize(deserializer)?;
    DateTime::from_timestamp_millis(timestamp)
        .ok_or_else(|| serde::de::Error::custom("Invalid timestamp"))
}
