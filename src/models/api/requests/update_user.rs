use crate::models::api::requests::validate_billing_type;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct UpdateUser {
    password: Option<String>,
    #[validate(custom(function = "validate_billing_type"))]
    pub billing: Option<String>,
    pub active: Option<bool>,
}

impl UpdateUser {
    pub fn password(&self) -> Option<String> {
        self.password
            .as_ref()
            .map(|p| bcrypt::hash(p, 10).ok())
            .flatten()
    }
}
