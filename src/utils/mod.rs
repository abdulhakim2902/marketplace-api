use uuid::Uuid;

pub mod date_utils;
pub mod object_utils;
pub mod schema;
pub mod shutdown_utils;
pub mod string_utils;
pub mod structs;
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

pub fn generate_marketplace_id(market_contract_address: &str, market_name: &str) -> Uuid {
    generate_uuid_from_str(format!("{}::{}", market_contract_address, market_name).as_str())
}

pub fn generate_attribute_id(
    collection_id: &str,
    nft_id: &str,
    attribute_type: &str,
    attribute_value: &str,
) -> Uuid {
    generate_uuid_from_str(
        format!(
            "{}::{}::{}::{}",
            collection_id, nft_id, attribute_type, attribute_value
        )
        .as_str(),
    )
}
pub fn generate_request_log_id(api_key_id: &str, ts: i64) -> Uuid {
    generate_uuid_from_str(format!("{}::{}", api_key_id, ts.to_string()).as_str())
}

pub fn generate_user_id(username: &str) -> Uuid {
    generate_uuid_from_str(username)
}
