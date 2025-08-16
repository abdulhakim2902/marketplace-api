pub mod api_key;
pub mod user;

use serde::Serialize;

#[derive(Serialize)]
pub struct HttpResponseConstErr {
    pub code: &'static str,
    pub msg: &'static str,
}

pub const UNAUTHORIZED_ERR: HttpResponseConstErr = HttpResponseConstErr {
    code: "ERR_401",
    msg: "Unauthorized",
};

#[derive(Serialize)]
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
