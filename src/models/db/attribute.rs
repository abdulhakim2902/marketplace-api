use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DbAttribute {
    pub collection_id: Option<String>,
    pub nft_id: Option<String>,
    pub attr_type: Option<String>,
    pub value: Option<String>,
}
