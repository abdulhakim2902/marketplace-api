pub mod access_token;
pub mod api_key;
pub mod auth_user;
pub mod user;

use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HttpResponseConstErr {
    pub code: &'static str,
    pub msg: &'static str,
}

pub const UNAUTHORIZED_ERR: HttpResponseConstErr = HttpResponseConstErr {
    code: "ERR_401",
    msg: "Unauthorized",
};

pub const BAD_REQUEST_ERR: HttpResponseConstErr = HttpResponseConstErr {
    code: "ERR_400",
    msg: "Bad Request",
};

#[derive(Serialize, ToSchema)]
pub struct HttpResponseErr {
    pub code: String,
    pub msg: String,
}

impl HttpResponseErr {
    pub fn new(code: &str, msg: &str) -> Self {
        Self {
            code: code.to_string(),
            msg: msg.to_string(),
        }
    }
}
