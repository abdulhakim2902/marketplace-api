use std::collections::HashMap;

use aptos_indexer_processor_sdk::utils::convert::standardize_address;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::workers::steps::marketplace::HashableJsonPath;

// event_type -> json_path, db_column
pub type EventFieldRemappings = HashMap<EventType, HashMap<HashableJsonPath, Vec<DbColumn>>>;
// resource_type -> json_path, db_column
pub type ResourceFieldRemappings = HashMap<String, HashMap<HashableJsonPath, Vec<DbColumn>>>;

pub type EventRemappingConfig = HashMap<String, EventRemapping>;
pub type ResourceRemappingConfig = HashMap<String, ResourceRemapping>;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct NFTMarketplaceConfig {
    pub name: String,
    pub starting_version: i64,
    pub contract_address: String,
    #[serde(default)]
    pub event_model_mapping: HashMap<String, MarketplaceEventType>,
    #[serde(default)]
    pub events: EventRemappingConfig,
    #[serde(default)]
    pub resources: ResourceRemappingConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbColumn {
    pub table: String,
    pub column: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct EventRemapping {
    pub event_fields: HashMap<String, Vec<DbColumn>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ResourceRemapping {
    pub resource_fields: HashMap<String, Vec<DbColumn>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, EnumString, Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum MarketplaceEventType {
    // Token event
    Mint,
    Burn,
    Transfer,
    Deposit,
    // Listing events
    List,
    Unlist,
    Buy,
    // Token bid events
    SoloBid,
    UnlistBid,
    AcceptBid,
    // Collection bid events
    CollectionBid,
    CancelCollectionBid,
    AcceptCollectionBid,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventType {
    address: String,
    module: String,
    r#struct: String,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}::{}::{}", self.address, self.module, self.r#struct)
    }
}

impl TryFrom<&str> for EventType {
    type Error = anyhow::Error;

    fn try_from(event_type: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = event_type.split("::").collect();
        if parts.len() < 3 {
            // With v1 events it is possible to emit primitives as events, e.g. just
            // emit an address or u64 as an event. We don't support this.
            anyhow::bail!("Unsupported event type: {}", event_type);
        }

        Ok(EventType {
            address: standardize_address(parts[0]),
            module: parts[1].to_string(),
            r#struct: parts[2..].join("::"), // Don't need to standardize generics because we won't support them
        })
    }
}

impl EventType {
    /// Returns true if the event type is a framework event. We don't always allow
    /// users to index framework events.
    //
    // WARNING: This code is only safe because we `standardize_address` in the
    // `try_from` implementation. If we add another way to instantiate an `EventType`,
    // it must also do this conversion.
    //
    // TODO: If we ever get a better Rust SDK, use AccountAddress instead.
    pub fn is_framework_event(&self) -> bool {
        // Convert address string to bytes. Skip "0x" prefix.
        let addr_bytes = hex::decode(&self.address[2..]).unwrap();

        // This is taken from AccountAddress::is_special.
        addr_bytes[..32 - 1].iter().all(|x| *x == 0) && addr_bytes[32 - 1] < 0b10000
    }

    pub fn get_struct(&self) -> &str {
        &self.r#struct
    }
}
