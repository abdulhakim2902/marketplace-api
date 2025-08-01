use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DataPointSchema {
    pub x: Option<DateTime<Utc>>,
    pub y: Option<i64>,
}

#[async_graphql::Object]
impl DataPointSchema {
    async fn x(&self) -> Option<String> {
        self.x.as_ref().map(|e| e.to_string())
    }

    async fn y(&self) -> Option<i64> {
        self.y
    }
}
