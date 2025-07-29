use crate::{
    config::marketplace_config::MarketplaceEventType,
    models::{
        EventModel,
        db::{activity::Activity, collection::Collection, nft::Nft},
        resources::{FromWriteResource, V2TokenResource},
    },
    utils::{
        object_utils::{ObjectAggregatedData, ObjectWithMetadata},
        token_utils::{TableMetadataForToken, TokenEvent},
    },
};
use ahash::AHashMap;
use aptos_indexer_processor_sdk::{
    aptos_indexer_transaction_stream::utils::time::parse_timestamp,
    aptos_protos::transaction::v1::{Transaction, transaction::TxnData, write_set_change::Change},
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::{convert::standardize_address, errors::ProcessorError},
};

pub struct TokenExtractor
where
    Self: Sized + Send + 'static,
{
    current_collections: AHashMap<String, Collection>,
    current_nfts: AHashMap<String, Nft>,
    current_burn_nfts: AHashMap<String, Nft>,
    current_activities: AHashMap<i64, Activity>,
}

impl TokenExtractor {
    pub fn new() -> Self {
        Self {
            current_collections: AHashMap::new(),
            current_nfts: AHashMap::new(),
            current_burn_nfts: AHashMap::new(),
            current_activities: AHashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl Processable for TokenExtractor {
    type Input = Vec<Transaction>;
    type Output = (Vec<Activity>, Vec<Collection>, Vec<Nft>);
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

                let mut deposit_event_owner: AHashMap<String, Option<String>> = AHashMap::new();
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
                                deposit_event_owner
                                    .insert(inner.id.token_data_id.to_addr(), account_address);
                            }
                            TokenEvent::TokenDeposit(inner) => {
                                deposit_event_owner
                                    .insert(inner.id.token_data_id.to_addr(), account_address);
                            }
                            _ => {}
                        }
                    }
                }

                for (event_index, event) in events.iter().enumerate() {
                    let event_model = EventModel::from_event(
                        event,
                        txn_version,
                        txn_block_height,
                        event_index as i64,
                        txn_ts,
                    )
                    .map_err(|e| ProcessorError::ProcessError {
                        message: format!("{e:#}"),
                    })?;

                    if let Some(event) = event_model {
                        let action_v1 =
                            Activity::get_action_from_token_event_v1(&event, &txn_id, txn_version)
                                .unwrap();

                        if let Some(mut activity) = action_v1 {
                            let tx_type = activity.tx_type.as_ref().unwrap().to_string();
                            if tx_type == MarketplaceEventType::Burn.to_string() {
                                if let Some(nft) =
                                    self.current_nfts.get_mut(activity.nft_id.as_ref().unwrap())
                                {
                                    nft.burned = Some(true);
                                    nft.owner = None;
                                } else {
                                    let nft: Nft = activity.clone().into();
                                    self.current_burn_nfts.insert(nft.id.clone(), nft);
                                }
                            }

                            if tx_type == MarketplaceEventType::Mint.to_string() {
                                if activity.receiver.is_none() {
                                    if let Some(owner) =
                                        deposit_event_owner.get(activity.nft_id.as_ref().unwrap())
                                    {
                                        activity.receiver = owner.clone();
                                    }
                                }
                            }

                            self.current_activities.insert(activity.tx_index, activity);
                        }

                        let action_v2 = Activity::get_action_from_token_event_v2(
                            &event,
                            &txn_id,
                            txn_version,
                            &token_metadata_helper,
                            sender.as_ref(),
                        )
                        .unwrap();

                        if let Some(activity) = action_v2 {
                            let tx_type = activity.tx_type.as_ref().unwrap().to_string();
                            if tx_type == MarketplaceEventType::Burn.to_string() {
                                if let Some(nft) =
                                    self.current_nfts.get_mut(activity.nft_id.as_ref().unwrap())
                                {
                                    nft.burned = Some(true);
                                    nft.owner = None;
                                } else {
                                    let nft: Nft = activity.clone().into();
                                    self.current_burn_nfts.insert(nft.id.clone(), nft);
                                }
                            }

                            self.current_activities.insert(activity.tx_index, activity);
                        }
                    }
                }

                for wsc in txn_info.changes.iter() {
                    match wsc.change.as_ref().unwrap() {
                        Change::WriteTableItem(table_item) => {
                            let collection_result = Collection::get_from_write_table_item(
                                table_item,
                                txn_version,
                                &table_handler_to_owner,
                            )
                            .unwrap();

                            if let Some(collection) = collection_result {
                                self.current_collections
                                    .insert(collection.id.clone(), collection);
                            }

                            let nft_result = Nft::get_from_write_table_item(
                                table_item,
                                txn_version,
                                &table_handler_to_owner,
                                &deposit_event_owner,
                            )
                            .unwrap();

                            if let Some(mut nft) = nft_result {
                                let burned_nft = self.current_burn_nfts.remove(nft.id.as_str());
                                if let Some(_) = burned_nft {
                                    nft.burned = Some(true);
                                    nft.owner = None;
                                }
                                self.current_nfts.insert(nft.id.clone(), nft);
                            }
                        }
                        Change::WriteResource(resource) => {
                            let colletion_result = Collection::get_from_write_resource(
                                resource,
                                &token_metadata_helper,
                            )
                            .unwrap();

                            if let Some(collection) = colletion_result {
                                self.current_collections
                                    .insert(collection.id.clone(), collection);
                            }

                            let nft_result =
                                Nft::get_from_write_resource(resource, &token_metadata_helper)
                                    .unwrap();

                            if let Some(mut nft) = nft_result {
                                let burned_nft = self.current_burn_nfts.remove(nft.id.as_str());
                                if let Some(_) = burned_nft {
                                    nft.burned = Some(true);
                                    nft.owner = None;
                                }

                                self.current_nfts.insert(nft.id.clone(), nft);
                            }
                        }
                        _ => {}
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

impl AsyncStep for TokenExtractor {}

impl NamedStep for TokenExtractor {
    fn name(&self) -> String {
        "TokenExtractorStep".to_string()
    }
}

impl TokenExtractor {
    fn drain(&mut self) -> (Vec<Activity>, Vec<Collection>, Vec<Nft>) {
        let mut nfts = self
            .current_nfts
            .drain()
            .map(|(_, v)| v)
            .collect::<Vec<Nft>>();

        let burn_nfts = self
            .current_burn_nfts
            .drain()
            .map(|(_, v)| v)
            .collect::<Vec<Nft>>();

        nfts.extend(burn_nfts);

        (
            self.current_activities.drain().map(|(_, v)| v).collect(),
            self.current_collections.drain().map(|(_, v)| v).collect(),
            nfts,
        )
    }
}
