use serde::Deserialize;
use validator::Validate;

use crate::models::api::requests::TimeRange;

#[derive(Deserialize, Validate, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloorChart {
    #[serde(flatten)]
    #[validate(nested)]
    pub time_range: TimeRange,
}
