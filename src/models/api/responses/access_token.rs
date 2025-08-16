use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
}
