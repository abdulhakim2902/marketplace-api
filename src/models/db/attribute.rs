use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DbAttribute {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub nft_id: Uuid,
    pub attr_type: String,
    pub value: String,
}
