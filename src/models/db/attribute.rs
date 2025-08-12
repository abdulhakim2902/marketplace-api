use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DbAttribute {
    pub collection_id: Option<String>,
    pub nft_id: Uuid,
    pub attr_type: Option<String>,
    pub value: Option<String>,
}
