use async_graphql::InputObject;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
