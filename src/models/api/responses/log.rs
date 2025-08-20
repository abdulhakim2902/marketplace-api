use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserLogSummaryResponse {
    pub api_key_id: Uuid,
    pub total: Option<i64>,
}
