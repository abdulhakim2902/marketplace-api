use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct DbProcessorStatus {
    pub processor: String,
    pub last_success_version: i64,
    pub last_transaction_timestamp: Option<DateTime<Utc>>,
}
