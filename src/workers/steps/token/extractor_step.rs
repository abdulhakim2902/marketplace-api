use std::sync::Arc;

use crate::database::IDatabase;
use crate::utils::generate_nft_id;
use crate::{
    config::marketplace_config::MarketplaceEventType,
    models::{
        EventModel,
        db::{activity::DbActivity, collection::DbCollection, nft::DbNft},
        resources::{FromWriteResource, V2TokenResource},
    },
    utils::{
        object_utils::{ObjectAggregatedData, ObjectWithMetadata},
        token_utils::{CoinEvent, TableMetadataForToken, TokenEvent},
    },
};
use ahash::{AHashMap, AHashSet};
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_indexer_transaction_stream::utils::time::parse_timestamp,
    aptos_protos::transaction::v1::{Transaction, transaction::TxnData, write_set_change::Change},
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::{convert::standardize_address, errors::ProcessorError},
};
use bigdecimal::{BigDecimal, ToPrimitive};
use uuid::Uuid;

pub struct TokenExtractor<TDb: IDatabase>
where
    Self: Sized + Send + 'static,
{
    db: Arc<TDb>,
    current_wallets: AHashSet<String>,
    current_collections: AHashMap<Uuid, DbCollection>,
    current_nfts: AHashMap<Uuid, DbNft>,
    current_burn_nfts: AHashMap<Uuid, DbNft>,
    current_activities: AHashMap<i64, DbActivity>,
}

impl<TDb: IDatabase> TokenExtractor<TDb> {
    pub fn new(db: Arc<TDb>) -> Self {
        Self {
            db,
            current_wallets: AHashSet::new(),
            current_collections: AHashMap::new(),
            current_nfts: AHashMap::new(),
            current_burn_nfts: AHashMap::new(),
            current_activities: AHashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl<TDb: IDatabase> Processable for TokenExtractor<TDb>
where
    TDb: Send + Sync,
{
    type Input = Vec<Transaction>;
    type Output = (Vec<DbActivity>, Vec<DbCollection>, Vec<DbNft>, Vec<String>);
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        transactions: TransactionContext<Vec<Transaction>>,
    ) -> Result<Option<TransactionContext<Self::Output>>, ProcessorError> {
        let mut token_metadata_helper: AHashMap<String, ObjectAggregatedData> = AHashMap::new();
        let table_handler_to_owner =
            TableMetadataForToken::get_table_handle_to_owner_from_transactions(&transactions.data);

        for txn in &transactions.data {
            if let Some(txn_info) = txn.info.as_ref() {
                let txn_id = format!("0x{}", hex::encode(txn_info.hash.as_slice()));
                let txn_version = txn.version as i64;
                let txn_block_height = txn.block_height as i64;
                let txn_ts =
                    parse_timestamp(txn.timestamp.as_ref().unwrap(), txn_version).naive_utc();

                let txn_data = match txn.txn_data.as_ref() {
                    Some(data) => data,
                    None => continue,
                };

                let default = vec![];
                let events = match txn_data {
                    TxnData::User(txn_inner) => txn_inner.events.as_slice(),
                    _ => &default,
                };

                let sender = match txn_data {
                    TxnData::User(txn_inner) => {
                        txn_inner.request.as_ref().map(|e| e.sender.to_string())
                    }
                    _ => None,
                };

                for wsc in txn_info.changes.iter() {
                    if let Change::WriteResource(wr) = wsc.change.as_ref().unwrap() {
                        if let Some(object) = ObjectWithMetadata::from_write_resource(wr).unwrap() {
                            token_metadata_helper.insert(
                                standardize_address(&wr.address),
                                ObjectAggregatedData {
                                    object,
                                    ..ObjectAggregatedData::default()
                                },
                            );
                        }
                    }
                }

                for wsc in txn_info.changes.iter() {
                    if let Change::WriteResource(wr) = wsc.change.as_ref().unwrap() {
                        let address = standardize_address(&wr.address.to_string());
                        if let Some(aggregated_data) = token_metadata_helper.get_mut(&address) {
                            let token_resource = V2TokenResource::from_write_resource(wr).unwrap();
                            if let Some(token_resource) = token_resource {
                                match token_resource {
                                    V2TokenResource::FixedSupply(fixed_supply) => {
                                        aggregated_data.fixed_supply = Some(fixed_supply);
                                    }
                                    V2TokenResource::ConcurrentySupply(concurrent_supply) => {
                                        aggregated_data.concurrent_supply = Some(concurrent_supply);
                                    }
                                    V2TokenResource::UnlimitedSupply(unlimited_supply) => {
                                        aggregated_data.unlimited_supply = Some(unlimited_supply);
                                    }
                                    V2TokenResource::TokenIdentifiers(token_identifiers) => {
                                        aggregated_data.token_identifiers = Some(token_identifiers);
                                    }
                                    V2TokenResource::Token(token) => {
                                        aggregated_data.token = Some(token);
                                    }
                                    V2TokenResource::PropertyMapModel(property_map) => {
                                        aggregated_data.property_map = Some(property_map);
                                    }
                                    V2TokenResource::Royalty(royalty) => {
                                        aggregated_data.royalty = Some(royalty);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }

                let mut token_owner: AHashMap<Uuid, Option<String>> = AHashMap::new();
                for event in events.iter() {
                    let token_event = TokenEvent::from_event(
                        event.type_str.as_ref(),
                        event.data.as_str(),
                        txn_version,
                    );

                    if let Some(token_event) = token_event.unwrap() {
                        let account_address = event
                            .key
                            .as_ref()
                            .map(|key| standardize_address(&key.account_address));

                        match token_event {
                            TokenEvent::DepositTokenEvent(inner) => {
                                token_owner.insert(
                                    generate_nft_id(inner.id.token_data_id.to_addr().as_str()),
                                    account_address,
                                );
                            }
                            TokenEvent::TokenDeposit(inner) => {
                                token_owner.insert(
                                    generate_nft_id(inner.id.token_data_id.to_addr().as_str()),
                                    account_address,
                                );
                            }
                            _ => {}
                        }
                    }
                }

                let mut mint_prices: AHashMap<String, BigDecimal> = AHashMap::new();
                let mut mint_activities: AHashMap<String, DbActivity> = AHashMap::new();
                for (event_index, event) in events.iter().enumerate() {
                    let event_model = EventModel::from_event(
                        event,
                        txn_version,
                        txn_block_height,
                        event_index as i64,
                        txn_ts,
                    );

                    if let Some(event) = event_model.unwrap() {
                        let coin = CoinEvent::from_event(
                            &event.type_str,
                            &event.data.to_string(),
                            txn_version,
                        );

                        if let Some(coin) = coin.unwrap() {
                            if let CoinEvent::WithdrawEvent(withdraw) = coin {
                                let mint_index = mint_activities
                                    .get(&event.account_address)
                                    .map(|e| e.tx_index);

                                if let Some(idx) = mint_index {
                                    if let Some(activity) = self.current_activities.get_mut(&idx) {
                                        let price = activity
                                            .price
                                            .as_ref()
                                            .map_or(withdraw.amount.clone(), |price| {
                                                price + &withdraw.amount
                                            });

                                        activity.price = price.to_i64();
                                    }
                                } else {
                                    mint_prices
                                        .entry(event.account_address.clone())
                                        .and_modify(|existing| {
                                            *existing += &withdraw.amount;
                                        })
                                        .or_insert(withdraw.amount);
                                }
                            }
                        }

                        let collection =
                            DbCollection::get_from_create_token_event(&event, txn_version);
                        if let Some(collection) = collection.unwrap() {
                            self.current_collections
                                .insert(collection.id.clone(), collection);
                        }

                        let activity_token_v1 = DbActivity::get_action_from_token_event_v1(
                            &event,
                            &txn_id,
                            txn_version,
                        );

                        if let Some(activity) = activity_token_v1.unwrap() {
                            self.merge_update(
                                activity,
                                &mut mint_prices,
                                &mut mint_activities,
                                &token_owner,
                            );
                        }

                        let activity_token_v2 = DbActivity::get_action_from_token_event_v2(
                            &event,
                            &txn_id,
                            txn_version,
                            &token_metadata_helper,
                            sender.as_ref(),
                        );

                        if let Some(activity) = activity_token_v2.unwrap() {
                            self.merge_update(
                                activity,
                                &mut mint_prices,
                                &mut mint_activities,
                                &token_owner,
                            );
                        }
                    }
                }

                for wsc in txn_info.changes.iter() {
                    let (collection_result, nft_result) = match wsc.change.as_ref().unwrap() {
                        Change::WriteTableItem(table_item) => {
                            let collection_result = DbCollection::get_from_write_table_item(
                                table_item,
                                txn_version,
                                &table_handler_to_owner,
                            );

                            let nft_result = DbNft::get_from_write_table_item(
                                table_item,
                                txn_version,
                                &table_handler_to_owner,
                                &token_owner,
                            );

                            (collection_result.unwrap(), nft_result.unwrap())
                        }
                        Change::WriteResource(resource) => {
                            let collection_result = DbCollection::get_from_write_resource(
                                resource,
                                &token_metadata_helper,
                            );

                            let nft_result =
                                DbNft::get_from_write_resource(resource, &token_metadata_helper);

                            (collection_result.unwrap(), nft_result.unwrap())
                        }
                        _ => (None, None),
                    };

                    if let Some(collection) = collection_result {
                        self.current_collections
                            .insert(collection.id.clone(), collection);
                    }

                    if let Some(mut nft) = nft_result {
                        let burned_nft = self.current_burn_nfts.remove(&nft.id);
                        if let Some(_) = burned_nft {
                            nft.burned = Some(true);
                            nft.owner = None;
                        }
                        self.current_nfts.insert(nft.id.clone(), nft);
                    }
                }
            }
        }

        let reduced_data = self.drain();

        Ok(Some(TransactionContext {
            data: reduced_data,
            metadata: transactions.metadata,
        }))
    }
}

impl<TDb: IDatabase> AsyncStep for TokenExtractor<TDb> {}

impl<TDb: IDatabase> NamedStep for TokenExtractor<TDb> {
    fn name(&self) -> String {
        "TokenExtractorStep".to_string()
    }
}

impl<TDb: IDatabase> TokenExtractor<TDb> {
    fn drain(&mut self) -> (Vec<DbActivity>, Vec<DbCollection>, Vec<DbNft>, Vec<String>) {
        let mut nfts = self
            .current_nfts
            .drain()
            .map(|(_, v)| v)
            .collect::<Vec<DbNft>>();

        let burn_nfts = self
            .current_burn_nfts
            .drain()
            .map(|(_, v)| v)
            .collect::<Vec<DbNft>>();

        nfts.extend(burn_nfts);

        (
            self.current_activities.drain().map(|(_, v)| v).collect(),
            self.current_collections.drain().map(|(_, v)| v).collect(),
            nfts,
            self.current_wallets.drain().map(|v| v).collect(),
        )
    }

    fn merge_update(
        &mut self,
        activity: DbActivity,
        mint_prices: &mut AHashMap<String, BigDecimal>,
        mint_activities: &mut AHashMap<String, DbActivity>,
        token_owner: &AHashMap<Uuid, Option<String>>,
    ) {
        let mut activity = activity;

        let tx_type = activity.tx_type.as_ref().unwrap().to_string();
        if tx_type == MarketplaceEventType::Burn.to_string() {
            if let Some(nft) = self.current_nfts.get_mut(activity.nft_id.as_ref().unwrap()) {
                nft.burned = Some(true);
                nft.owner = None;
            } else {
                let nft_result: Result<DbNft> = activity.clone().try_into();
                if let Ok(nft) = nft_result {
                    self.current_burn_nfts.insert(nft.id.clone(), nft);
                }
            }
        }

        if tx_type == MarketplaceEventType::Mint.to_string() {
            if activity.receiver.is_none() {
                if let Some(nft_id) = activity.nft_id.as_ref() {
                    if let Some(owner) = token_owner.get(nft_id) {
                        activity.receiver = owner.clone();
                    }
                }
            }

            if let Some(receiver) = activity.receiver.as_ref() {
                if let Some(price) = mint_prices.remove(receiver).as_ref() {
                    activity.price = price.to_i64();
                } else {
                    mint_activities.insert(receiver.to_string(), activity.clone());
                }
            }
        }

        if let Some(address) = activity.sender.as_ref() {
            self.current_wallets.insert(address.to_string());
        }

        if let Some(address) = activity.receiver.as_ref() {
            self.current_wallets.insert(address.to_string());
        }

        self.current_activities.insert(activity.tx_index, activity);
    }
}
