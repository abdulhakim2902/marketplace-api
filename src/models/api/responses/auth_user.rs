use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct AuthUserResponse {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub role: String,
}
