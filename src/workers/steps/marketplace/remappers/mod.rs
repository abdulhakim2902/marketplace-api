pub mod event_remapper;
pub mod resource_remapper;

use crate::models::marketplace::NFT_MARKETPLACE_ACTIVITIES_TABLE_NAME;

#[derive(Debug, PartialEq, Eq)]
enum TableType {
    Activities,
}

impl TableType {
    fn from_str(table_name: &str) -> Option<Self> {
        match table_name {
            NFT_MARKETPLACE_ACTIVITIES_TABLE_NAME => Some(TableType::Activities),
            _ => None,
        }
    }
}
