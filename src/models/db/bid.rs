use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DbBid {
    pub id: Uuid,
    pub bidder: Option<String>,
    pub accepted_tx_id: Option<String>,
    pub cancelled_tx_id: Option<String>,
    pub collection_id: Option<Uuid>,
    pub created_tx_id: Option<String>,
    pub expired_at: Option<NaiveDateTime>,
    pub market_contract_id: Option<String>,
    pub market_name: Option<String>,
    pub nonce: Option<String>,
    pub nft_id: Option<Uuid>,
    pub price: Option<i64>,
    pub receiver: Option<String>,
    pub remaining_count: Option<i64>,
    pub status: Option<String>,
    pub bid_type: Option<String>,
}
