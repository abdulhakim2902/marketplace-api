use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    password: String,
    pub billing: String,
}

impl CreateUser {
    pub fn password(&self) -> bcrypt::BcryptResult<String> {
        bcrypt::hash(&self.password, bcrypt::DEFAULT_COST)
    }
}
