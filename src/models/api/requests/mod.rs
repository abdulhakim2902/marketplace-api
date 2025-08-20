pub mod create_api_key;
pub mod create_user;
pub mod login;
pub mod update_user;

use validator::ValidationError;

pub fn validate_billing_type(billing: &str) -> Result<(), ValidationError> {
    let billing_types = ["per_call", "flat_fee"];
    if billing_types.contains(&billing) {
        Ok(())
    } else {
        Err(ValidationError::new(
            "Invalid billing type. Only accept 'per_call' or 'flat_fee'",
        ))
    }
}
