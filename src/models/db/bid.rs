use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Bid {
    pub id: Option<String>,
    pub bidder: Option<String>,
    pub accepted_tx_id: Option<String>,
    pub canceled_tx_id: Option<String>,
    pub collection_id: Option<String>,
    pub created_tx_id: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub market_contract_id: Option<String>,
    pub market_name: Option<String>,
    pub nonce: Option<String>,
    pub nft_id: Option<String>,
    pub price: Option<i64>,
    pub price_str: Option<String>,
    pub receiver: Option<String>,
    pub remaining_count: Option<i64>,
    pub status: Option<String>,
    pub bid_type: Option<String>,
}
