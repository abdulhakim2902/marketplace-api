use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateApiKey {
    pub name: String,
    pub description: Option<String>,
}
