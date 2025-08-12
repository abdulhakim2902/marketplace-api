use uuid::Uuid;

pub mod date_utils;
pub mod object_utils;
pub mod shutdown_utils;
pub mod string_utils;
pub mod token_utils;

pub fn generate_uuid_from_str(value: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, value.as_bytes())
}

pub fn generate_activity_id(tx_index: i64) -> Uuid {
    generate_uuid_from_str(tx_index.to_string().as_str())
}

pub fn generate_collection_id(collection: &str) -> Uuid {
    generate_uuid_from_str(collection)
}

pub fn generate_nft_id(token_id: &str) -> Uuid {
    generate_uuid_from_str(token_id)
}

pub fn generate_listing_id(market_contract_address: &str, token_id: &str) -> Uuid {
    generate_uuid_from_str(format!("{}::{}", market_contract_address, token_id).as_str())
}

pub fn generate_bid_id(market_contract_address: &str, token_id: &str, bidder: &str) -> Uuid {
    generate_uuid_from_str(
        format!("{}::{}::{}", market_contract_address, token_id, bidder).as_str(),
    )
}
