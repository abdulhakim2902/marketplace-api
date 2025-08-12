use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DbListing {
    pub id: Uuid,
    pub block_height: Option<i64>,
    pub block_time: Option<NaiveDateTime>,
    pub market_contract_id: Option<String>,
    pub listed: Option<bool>,
    pub market_name: Option<String>,
    pub collection_id: Option<String>,
    pub nft_id: Option<String>,
    pub nonce: Option<String>,
    pub price: Option<i64>,
    pub seller: Option<String>,
    pub tx_index: Option<i64>,
}
