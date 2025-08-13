use crate::utils::{generate_activity_id, generate_collection_id, generate_nft_id};
use crate::{
    config::marketplace_config::MarketplaceEventType,
    models::{EventModel, db::nft::DbNft},
    utils::{
        object_utils::ObjectAggregatedData,
        token_utils::{TokenEvent, V2TokenEvent},
    },
};
use ahash::AHashMap;
use anyhow::Context;
use aptos_indexer_processor_sdk::utils::convert::standardize_address;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DbActivity {
    pub id: Uuid,
    pub tx_index: i64,
    pub tx_id: String,
    pub tx_type: Option<String>,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub price: Option<i64>,
    pub nft_id: Option<Uuid>,
    pub collection_id: Option<Uuid>,
    pub block_time: Option<NaiveDateTime>,
    pub block_height: Option<i64>,
    pub market_name: Option<String>,
    pub market_contract_id: Option<String>,
    pub usd_price: Option<BigDecimal>,
    pub amount: Option<i64>,
}

impl DbActivity {
    pub fn get_action_from_token_event_v1(
        event: &EventModel,
        txn_id: &str,
        txn_version: i64,
    ) -> anyhow::Result<Option<Self>> {
        let event_type = event.type_str.clone();
        let token_event = TokenEvent::from_event(&event_type, &event.data.to_string(), txn_version);
        if let Some(token_event) = token_event? {
            let token_activity = match &token_event {
                TokenEvent::Mint(inner) => {
                    let collection_id =
                        generate_collection_id(inner.id.get_collection_addr().as_str());
                    let nft_id = generate_collection_id(inner.id.to_addr().as_str());

                    Some(DbActivity {
                        id: generate_activity_id(event.get_tx_index()),
                        tx_id: txn_id.to_string(),
                        tx_index: event.get_tx_index(),
                        block_time: Some(event.block_timestamp),
                        block_height: Some(event.transaction_block_height),
                        tx_type: Some(MarketplaceEventType::Mint.to_string()),
                        collection_id: Some(collection_id),
                        nft_id: Some(nft_id),
                        amount: Some(1),
                        ..Default::default()
                    })
                }
                TokenEvent::MintTokenEvent(inner) => {
                    let collection_id =
                        generate_collection_id(inner.id.get_collection_addr().as_str());
                    let nft_id = generate_collection_id(inner.id.to_addr().as_str());

                    Some(DbActivity {
                        id: generate_activity_id(event.get_tx_index()),
                        tx_id: txn_id.to_string(),
                        tx_index: event.get_tx_index(),
                        block_time: Some(event.block_timestamp),
                        block_height: Some(event.transaction_block_height),
                        tx_type: Some(MarketplaceEventType::Mint.to_string()),
                        collection_id: Some(collection_id),
                        amount: Some(1),
                        nft_id: Some(nft_id),
                        ..Default::default()
                    })
                }
                TokenEvent::Burn(inner) => {
                    let collection_id = generate_collection_id(
                        inner.id.token_data_id.get_collection_addr().as_str(),
                    );
                    let nft_id = generate_collection_id(inner.id.token_data_id.to_addr().as_str());

                    Some(DbActivity {
                        id: generate_activity_id(event.get_tx_index()),
                        tx_id: txn_id.to_string(),
                        tx_index: event.get_tx_index(),
                        block_time: Some(event.block_timestamp),
                        block_height: Some(event.transaction_block_height),
                        tx_type: Some(MarketplaceEventType::Burn.to_string()),
                        sender: Some(inner.get_account()),
                        collection_id: Some(collection_id),
                        nft_id: Some(nft_id),
                        amount: Some(1),
                        ..Default::default()
                    })
                }
                TokenEvent::BurnTokenEvent(inner) => {
                    let collection_id = generate_collection_id(
                        inner.id.token_data_id.get_collection_addr().as_str(),
                    );
                    let nft_id = generate_collection_id(inner.id.token_data_id.to_addr().as_str());

                    Some(DbActivity {
                        id: generate_activity_id(event.get_tx_index()),
                        tx_id: txn_id.to_string(),
                        tx_index: event.get_tx_index(),
                        block_time: Some(event.block_timestamp),
                        block_height: Some(event.transaction_block_height),
                        tx_type: Some(MarketplaceEventType::Burn.to_string()),
                        sender: Some(standardize_address(&event.account_address)),
                        collection_id: Some(collection_id),
                        nft_id: Some(nft_id),
                        amount: Some(1),
                        ..Default::default()
                    })
                }
                _ => None,
            };

            return Ok(token_activity);
        }

        Ok(None)
    }

    pub fn get_action_from_token_event_v2(
        event: &EventModel,
        txn_id: &str,
        txn_version: i64,
        object_metadata: &AHashMap<String, ObjectAggregatedData>,
        sender: Option<&String>,
    ) -> anyhow::Result<Option<Self>> {
        let event_type = event.type_str.clone();
        let token_event =
            V2TokenEvent::from_event(&event_type, &event.data.to_string(), txn_version);

        if let Some(token_event) = token_event? {
            let token_addr = match &token_event {
                V2TokenEvent::MintEvent(inner) => inner.get_token_address(),
                V2TokenEvent::Mint(inner) => inner.get_token_address(),
                V2TokenEvent::BurnEvent(inner) => inner.get_token_address(),
                V2TokenEvent::Burn(inner) => inner.get_token_address(),
                V2TokenEvent::TransferEvent(inner) => inner.get_object_address(),
                _ => standardize_address(&event.account_address),
            };

            if let Some(object_data) = object_metadata.get(&token_addr) {
                let token_activity = match token_event {
                    V2TokenEvent::Mint(mint) => {
                        let collection_id =
                            generate_collection_id(mint.get_collection_address().as_str());
                        let nft_id = generate_nft_id(mint.get_token_address().as_str());

                        Some(DbActivity {
                            id: generate_activity_id(event.get_tx_index()),
                            tx_id: txn_id.to_string(),
                            tx_index: event.get_tx_index(),
                            block_height: Some(event.transaction_block_height),
                            block_time: Some(event.block_timestamp),
                            tx_type: Some(MarketplaceEventType::Mint.to_string()),
                            receiver: Some(object_data.object.object_core.get_owner_address()),
                            collection_id: Some(collection_id),
                            amount: Some(1),
                            nft_id: Some(nft_id),
                            ..Default::default()
                        })
                    }
                    V2TokenEvent::MintEvent(mint) => {
                        let collection_id =
                            generate_collection_id(&standardize_address(&event.account_address));
                        let nft_id = generate_nft_id(mint.get_token_address().as_str());

                        Some(DbActivity {
                            id: generate_activity_id(event.get_tx_index()),
                            tx_id: txn_id.to_string(),
                            tx_index: event.get_tx_index(),
                            block_height: Some(event.transaction_block_height),
                            block_time: Some(event.block_timestamp),
                            tx_type: Some(MarketplaceEventType::Mint.to_string()),
                            receiver: Some(object_data.object.object_core.get_owner_address()),
                            collection_id: Some(collection_id),
                            amount: Some(1),
                            nft_id: Some(nft_id),
                            ..Default::default()
                        })
                    }
                    V2TokenEvent::Burn(burn) => {
                        let collection_id =
                            generate_collection_id(burn.get_collection_address().as_str());
                        let nft_id = generate_nft_id(burn.get_token_address().as_str());

                        Some(DbActivity {
                            id: generate_activity_id(event.get_tx_index()),
                            tx_id: txn_id.to_string(),
                            tx_index: event.get_tx_index(),
                            block_height: Some(event.transaction_block_height),
                            block_time: Some(event.block_timestamp),
                            tx_type: Some(MarketplaceEventType::Burn.to_string()),
                            sender: burn.get_previous_owner_address(),
                            collection_id: Some(collection_id),
                            amount: Some(1),
                            nft_id: Some(nft_id),
                            ..Default::default()
                        })
                    }
                    V2TokenEvent::BurnEvent(burn) => {
                        let collection_id =
                            generate_collection_id(&standardize_address(&event.account_address));
                        let nft_id = generate_nft_id(burn.get_token_address().as_str());

                        Some(DbActivity {
                            id: generate_activity_id(event.get_tx_index()),
                            tx_id: txn_id.to_string(),
                            tx_index: event.get_tx_index(),
                            block_height: Some(event.transaction_block_height),
                            block_time: Some(event.block_timestamp),
                            tx_type: Some(MarketplaceEventType::Burn.to_string()),
                            sender: sender.map(|s| s.to_string()),
                            collection_id: Some(collection_id),
                            amount: Some(1),
                            nft_id: Some(nft_id),
                            ..Default::default()
                        })
                    }
                    V2TokenEvent::TransferEvent(transfer) => {
                        if let Some(token) = &object_data.token {
                            let collection_id =
                                generate_collection_id(token.get_collection_address().as_str());
                            let nft_id = generate_nft_id(transfer.get_object_address().as_str());

                            Some(DbActivity {
                                id: generate_activity_id(event.get_tx_index()),
                                tx_id: txn_id.to_string(),
                                tx_index: event.get_tx_index(),
                                block_height: Some(event.transaction_block_height),
                                block_time: Some(event.block_timestamp),
                                tx_type: Some(MarketplaceEventType::Transfer.to_string()),
                                sender: Some(transfer.get_from_address()),
                                receiver: Some(transfer.get_to_address()),
                                collection_id: Some(collection_id),
                                amount: Some(1),
                                nft_id: Some(nft_id),
                                ..Default::default()
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                return Ok(token_activity);
            }
        }

        Ok(None)
    }
}

impl TryFrom<DbActivity> for DbNft {
    type Error = anyhow::Error;

    fn try_from(value: DbActivity) -> anyhow::Result<Self> {
        let nft_id = value.nft_id.context("Invalid nft id")?;

        Ok(Self {
            id: nft_id,
            burned: Some(true),
            collection_id: value.collection_id,
            ..Default::default()
        })
    }
}
