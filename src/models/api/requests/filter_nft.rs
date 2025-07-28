use serde::Deserialize;
use validator::Validate;

use crate::models::api::requests::PagingRequest;

#[derive(Deserialize, Validate, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilterNft {
    #[serde(default)]
    pub account: String,
    #[serde(flatten)]
    pub paging: PagingRequest,
}
