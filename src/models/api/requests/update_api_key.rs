use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct UpdateApiKey {
    pub name: Option<String>,
    pub description: Option<String>,
}
