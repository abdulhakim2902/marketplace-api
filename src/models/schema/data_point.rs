use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, SimpleObject, ToSchema)]
#[graphql(name = "DataPoint")]
pub struct DataPointSchema {
    pub x: Option<DateTime<Utc>>,
    pub y: Option<i64>,
}
