use crate::models::api::requests::validate_billing_type;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateUser {
    pub username: String,
    password: String,
    #[validate(custom(function = "validate_billing_type"))]
    pub billing: String,
}

impl CreateUser {
    pub fn new(username: &str, password: &str, billing: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
            billing: billing.to_string(),
        }
    }
}

impl CreateUser {
    pub fn password(&self) -> bcrypt::BcryptResult<String> {
        bcrypt::hash(&self.password, 10)
    }
}
