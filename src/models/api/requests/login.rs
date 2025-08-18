use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct Login {
    pub username: String,
    pub password: String,
}
