use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DataPoint {
    pub x: Option<DateTime<Utc>>,
    pub y: Option<BigDecimal>,
}

#[async_graphql::Object]
impl DataPoint {
    async fn x(&self) -> Option<String> {
        self.x.as_ref().map(|e| e.to_string())
    }

    async fn y(&self) -> Option<String> {
        self.y.as_ref().map(|e| e.to_string())
    }
}
