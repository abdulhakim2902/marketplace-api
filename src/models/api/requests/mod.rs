use serde::Deserialize;
use serde_with::{DisplayFromStr, PickFirst, serde_as};
use validator::Validate;

pub mod filter_collection;

#[serde_as]
#[derive(Deserialize, Debug, Clone, Validate, Default)]
#[serde(rename_all = "camelCase")]
pub struct PagingRequest {
    #[validate(range(min = 1, message = "page must be greater than 0"))]
    #[serde_as(as = "PickFirst<(_, DisplayFromStr)>")]
    #[serde(default = "default_page")]
    pub page: i64,

    #[validate(range(min = 1, message = "page_size must be greater than 0"))]
    #[serde_as(as = "PickFirst<(_, DisplayFromStr)>")]
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}
