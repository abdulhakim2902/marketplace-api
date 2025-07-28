use serde::Deserialize;
use validator::Validate;

use crate::models::api::requests::PagingRequest;

#[derive(Deserialize, Validate, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilterNftHolder {
    #[serde(flatten)]
    pub paging: PagingRequest,
}
