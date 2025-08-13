use crate::utils::{
    generate_activity_id, generate_bid_id, generate_collection_id, generate_listing_id,
    generate_nft_id,
};
use crate::{
    config::marketplace_config::MarketplaceEventType,
    models::db::{activity::DbActivity, bid::DbBid, listing::DbListing},
};
use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDateTime};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;

pub const NFT_MARKETPLACE_ACTIVITIES_TABLE_NAME: &str = "nft_marketplace_activities";
pub const APT_DECIMAL: i32 = 100_000_000;

/**
 * NftMarketplaceActivity is the main model for storing NFT marketplace activities.
*/
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NftMarketplaceActivity {
    pub txn_id: String,
    pub txn_version: i64,
    pub index: i64,
    pub raw_event_type: String,
    pub standard_event_type: MarketplaceEventType,
    pub creator_address: Option<String>,
    pub collection_addr: Option<String>,
    pub collection_name: Option<String>,
    pub token_addr: Option<String>,
    pub token_name: Option<String>,
    pub price: i64,
    pub usd_price: Option<BigDecimal>,
    pub token_amount: Option<i64>,
    pub buyer: Option<String>,
    pub seller: Option<String>,
    pub listing_id: Option<String>,
    pub offer_id: Option<String>,
    pub json_data: serde_json::Value,
    pub marketplace: Option<String>,
    pub contract_address: Option<String>,
    pub block_timestamp: NaiveDateTime,
    pub block_height: i64,
    pub expiration_time: Option<i64>,
    pub bid_key: Option<i64>,
    pub start_time: Option<i64>,
    pub duration: Option<i64>,
}

impl From<NftMarketplaceActivity> for DbActivity {
    fn from(value: NftMarketplaceActivity) -> Self {
        Self {
            id: generate_activity_id(value.get_tx_index()),
            tx_index: value.get_tx_index(),
            price: Some(value.price),
            market_contract_id: value.contract_address,
            tx_id: value.txn_id,
            nft_id: value.token_addr.as_ref().map(|e| generate_nft_id(e)),
            tx_type: Some(value.standard_event_type.to_string()),
            collection_id: value
                .collection_addr
                .as_ref()
                .map(|e| generate_collection_id(e)),
            sender: value.seller,
            receiver: value.buyer,
            block_time: Some(value.block_timestamp),
            market_name: value.marketplace,
            block_height: Some(value.block_height),
            usd_price: value.usd_price,
            amount: value.token_amount,
        }
    }
}

impl TryFrom<NftMarketplaceActivity> for DbBid {
    type Error = anyhow::Error;

    fn try_from(value: NftMarketplaceActivity) -> anyhow::Result<Self> {
        Ok(Self {
            id: value.get_bid_id().context("Invalid bid")?,
            created_tx_id: value.get_created_txn_id(),
            accepted_tx_id: value.get_accepted_txn_id(),
            cancelled_tx_id: value.get_cancelled_txn_id(),
            bid_type: value.get_bid_type(),
            status: value.get_bid_status(),
            expired_at: value.get_expiration_time(),
            price: Some(value.price),
            market_contract_id: value.contract_address,
            market_name: value.marketplace,
            collection_id: value
                .collection_addr
                .as_ref()
                .map(|e| generate_collection_id(e)),
            nft_id: value.token_addr.as_ref().map(|e| generate_nft_id(e)),
            nonce: value.offer_id,
            bidder: value.buyer,
            remaining_count: value.token_amount,
            receiver: value.seller,
        })
    }
}

impl TryFrom<NftMarketplaceActivity> for DbListing {
    type Error = anyhow::Error;

    fn try_from(value: NftMarketplaceActivity) -> anyhow::Result<Self> {
        Ok(Self {
            id: value.get_listing_id().context("Invalid listing")?,
            tx_index: Some(value.get_tx_index()),
            listed: value.get_listing_status(),
            price: Some(value.price),
            market_contract_id: value.contract_address,
            collection_id: value
                .collection_addr
                .as_ref()
                .map(|e| generate_collection_id(e)),
            nft_id: value.token_addr.as_ref().map(|e| generate_nft_id(e)),
            market_name: value.marketplace,
            seller: value.seller,
            block_time: Some(value.block_timestamp),
            nonce: value.listing_id,
            block_height: Some(value.block_height),
        })
    }
}

impl NftMarketplaceActivity {
    pub fn get_tx_index(&self) -> i64 {
        self.txn_version * 100_000 + self.index
    }
}

impl MarketplaceModel for NftMarketplaceActivity {
    fn set_field(&mut self, field: MarketplaceField, value: String) {
        if value.is_empty() {
            tracing::debug!("Empty value for field: {:?}", field);
            return;
        }

        match field {
            MarketplaceField::CollectionAddr => self.collection_addr = Some(value),
            MarketplaceField::TokenAddr => self.token_addr = Some(value),
            MarketplaceField::TokenName => self.token_name = Some(value),
            MarketplaceField::CreatorAddress => self.creator_address = Some(value),
            MarketplaceField::CollectionName => self.collection_name = Some(value),
            MarketplaceField::Price => self.price = value.parse().unwrap_or(0),
            MarketplaceField::TokenAmount => self.token_amount = value.parse().ok(),
            MarketplaceField::Buyer => self.buyer = Some(value),
            MarketplaceField::Seller => self.seller = Some(value),
            MarketplaceField::StartTime => self.start_time = value.parse().ok(),
            MarketplaceField::Duration => self.duration = value.parse().ok(),
            MarketplaceField::ExpirationTime => self.expiration_time = value.parse().ok(),
            MarketplaceField::ListingId => self.listing_id = Some(value),
            MarketplaceField::OfferId | MarketplaceField::CollectionOfferId => {
                self.offer_id = Some(value)
            }
            MarketplaceField::Marketplace => self.marketplace = Some(value),
            MarketplaceField::ContractAddress => self.contract_address = Some(value),
            MarketplaceField::BlockTimestamp => {
                self.block_timestamp = value.parse().unwrap_or(NaiveDateTime::default())
            }
            MarketplaceField::BidKey => self.bid_key = value.parse().ok(),
            _ => tracing::debug!("Unknown field: {:?}", field),
        }
    }

    // This is a function that is used to check if we have all the necessary fields to insert the model into the database.
    // DbActivity table uses txn_version, index, and marketplace as the primary key, so it's rare that we need to check if it's valid.
    // So we use this function to check if has the contract_address and marketplace. to make sure we can easily filter out marketplaces that don't exist.
    // TODO: if we want to be more strict, we can have a whitelist of marketplaces that are allowed to be inserted into the database.
    fn is_valid(&self) -> bool {
        !self.marketplace.is_none() && !self.contract_address.is_none()
    }

    fn table_name(&self) -> &'static str {
        NFT_MARKETPLACE_ACTIVITIES_TABLE_NAME
    }

    fn updated_at(&self) -> i64 {
        self.block_timestamp.and_utc().timestamp()
    }

    fn get_field(&self, field: MarketplaceField) -> Option<String> {
        match field {
            MarketplaceField::CollectionAddr => self.collection_addr.clone(),
            MarketplaceField::TokenAddr => self.token_addr.clone(),
            MarketplaceField::TokenName => self.token_name.clone(),
            MarketplaceField::CreatorAddress => self.creator_address.clone(),
            MarketplaceField::CollectionName => self.collection_name.clone(),
            MarketplaceField::Price => Some(self.price.to_string()),
            MarketplaceField::TokenAmount => self.token_amount.map(|amount| amount.to_string()),
            MarketplaceField::Buyer => self.buyer.clone(),
            MarketplaceField::Seller => self.seller.clone(),
            MarketplaceField::ExpirationTime => self.expiration_time.map(|ts| ts.to_string()),
            MarketplaceField::ListingId => self.listing_id.clone(),
            MarketplaceField::OfferId => self.offer_id.clone(),
            MarketplaceField::Marketplace => self.marketplace.clone(),
            MarketplaceField::ContractAddress => self.contract_address.clone(),
            MarketplaceField::BlockTimestamp => Some(self.block_timestamp.to_string()),
            MarketplaceField::BidKey => self.bid_key.map(|val| val.to_string()),
            _ => None,
        }
    }

    fn get_txn_version(&self) -> i64 {
        self.txn_version
    }

    fn get_standard_event_type(&self) -> String {
        self.standard_event_type.to_string()
    }
}

impl BidModel for NftMarketplaceActivity {
    fn get_bid_id(&self) -> Option<Uuid> {
        if let Some(status) = self.get_bid_type().as_ref() {
            if let Some(bidder) = self.buyer.as_ref() {
                let address = match status.as_str() {
                    "solo" => self.token_addr.as_ref(),
                    "collection" => self.collection_addr.as_ref(),
                    _ => None,
                };

                return self.contract_address.as_ref().zip(address).map(
                    |(contract_address, address)| {
                        generate_bid_id(contract_address, address, bidder)
                    },
                );
            }
        }

        None
    }

    fn get_bid_status(&self) -> Option<String> {
        match self.standard_event_type {
            MarketplaceEventType::SoloBid | MarketplaceEventType::CollectionBid => {
                Some("active".to_string())
            }
            MarketplaceEventType::AcceptBid | MarketplaceEventType::AcceptCollectionBid => {
                Some("matched".to_string())
            }
            MarketplaceEventType::UnlistBid | MarketplaceEventType::CancelCollectionBid => {
                Some("cancelled".to_string())
            }
            _ => None,
        }
    }

    fn get_bid_type(&self) -> Option<String> {
        match self.standard_event_type {
            MarketplaceEventType::SoloBid
            | MarketplaceEventType::AcceptBid
            | MarketplaceEventType::UnlistBid => Some("solo".to_string()),
            MarketplaceEventType::CollectionBid
            | MarketplaceEventType::AcceptCollectionBid
            | MarketplaceEventType::CancelCollectionBid => Some("collection".to_string()),
            _ => None,
        }
    }

    fn get_created_txn_id(&self) -> Option<String> {
        match self.standard_event_type {
            MarketplaceEventType::SoloBid | MarketplaceEventType::CollectionBid => {
                Some(self.txn_id.clone())
            }
            _ => None,
        }
    }

    fn get_cancelled_txn_id(&self) -> Option<String> {
        match self.standard_event_type {
            MarketplaceEventType::UnlistBid | MarketplaceEventType::CancelCollectionBid => {
                Some(self.txn_id.clone())
            }
            _ => None,
        }
    }

    fn get_accepted_txn_id(&self) -> Option<String> {
        match self.standard_event_type {
            MarketplaceEventType::AcceptBid | MarketplaceEventType::AcceptCollectionBid => {
                Some(self.txn_id.clone())
            }
            _ => None,
        }
    }

    fn get_expiration_time(&self) -> Option<NaiveDateTime> {
        if let Some(expiration_time) = self.expiration_time {
            if let Some(marketplace) = self.marketplace.as_ref() {
                return match marketplace.as_str() {
                    "topaz" => {
                        DateTime::from_timestamp_micros(expiration_time).map(|e| e.naive_utc())
                    }
                    "rarible" => {
                        DateTime::from_timestamp(expiration_time, 0).map(|e| e.naive_utc())
                    }
                    _ => None,
                };
            }
        }

        let ts = self
            .start_time
            .zip(self.duration)
            .map(|(start_time, duration)| start_time + duration);

        if let Some(ts) = ts {
            return DateTime::from_timestamp_millis(ts).map(|e| e.naive_utc());
        }

        None
    }
}

impl ListingModel for NftMarketplaceActivity {
    fn get_listing_id(&self) -> Option<Uuid> {
        if self.get_listing_status().is_some() {
            return self
                .contract_address
                .as_ref()
                .zip(self.token_addr.as_ref())
                .map(|(contract_address, token_addr)| {
                    generate_listing_id(contract_address, token_addr)
                });
        }

        None
    }

    fn get_listing_status(&self) -> Option<bool> {
        match self.standard_event_type {
            MarketplaceEventType::List => Some(true),
            MarketplaceEventType::Relist => Some(true),
            MarketplaceEventType::Unlist => Some(false),
            MarketplaceEventType::Buy => Some(false),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum MarketplaceField {
    CollectionAddr,
    TokenAddr,
    TokenName,
    CreatorAddress,
    CollectionName,
    Price,
    TokenAmount,
    Buyer,
    Seller,
    ExpirationTime,
    ListingId,
    OfferId,
    CollectionOfferId,
    Marketplace,
    ContractAddress,
    LastTransactionVersion,
    LastTransactionTimestamp,
    RemainingTokenAmount,
    BlockTimestamp,
    BidKey,
    StartTime,
    Duration,
}

pub trait MarketplaceModel {
    fn set_field(&mut self, field: MarketplaceField, value: String);
    fn is_valid(&self) -> bool;
    fn table_name(&self) -> &'static str;
    fn updated_at(&self) -> i64;
    fn get_field(&self, field: MarketplaceField) -> Option<String>;
    fn get_txn_version(&self) -> i64;
    fn get_standard_event_type(&self) -> String;
}

pub trait BidModel {
    fn get_bid_id(&self) -> Option<Uuid>;
    fn get_bid_status(&self) -> Option<String>;
    fn get_bid_type(&self) -> Option<String>;
    fn get_created_txn_id(&self) -> Option<String>;
    fn get_cancelled_txn_id(&self) -> Option<String>;
    fn get_accepted_txn_id(&self) -> Option<String>;
    fn get_expiration_time(&self) -> Option<NaiveDateTime>;
}

pub trait ListingModel {
    fn get_listing_id(&self) -> Option<Uuid>;
    fn get_listing_status(&self) -> Option<bool>;
}
