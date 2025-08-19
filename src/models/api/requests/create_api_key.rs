use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateApiKey {
    pub name: String,
    pub description: Option<String>,
}
