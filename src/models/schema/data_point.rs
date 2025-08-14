use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, SimpleObject)]
pub struct DataPointSchema {
    pub x: Option<DateTime<Utc>>,
    pub y: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterFloorChartSchema {
    #[graphql(name = "where")]
    pub where_: WhereFloorChartSchema,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereFloorChartSchema {
    pub collection_id: String,
    pub start_time: i64,
    pub end_time: i64,
    pub interval: String,
}
