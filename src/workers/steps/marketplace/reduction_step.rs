use crate::{
    cache::ICache,
    database::{IDatabase, token_prices::ITokenPrices},
    models::{
        db::{
            activity::DbActivity, bid::DbBid, collection::DbCollection, listing::DbListing,
            nft::DbNft,
        },
        marketplace::{APT_DECIMAL, MarketplaceField, MarketplaceModel, NftMarketplaceActivity},
    },
    utils::string_utils::capitalize,
};
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use bigdecimal::BigDecimal;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use uuid::Uuid;

const APT_TOKEN_ADDR: &str = "0x000000000000000000000000000000000000000000000000000000000000000a";

#[derive(Clone, Debug, Default)]
pub struct NFTAccumulator {
    activities: HashMap<Uuid, DbActivity>,
    bids: HashMap<Uuid, DbBid>,
    listings: HashMap<Uuid, DbListing>,
    collections: HashMap<Uuid, DbCollection>,
    nfts: HashMap<Uuid, DbNft>,
}

impl NFTAccumulator {
    pub fn fold_activity(&mut self, activity: &NftMarketplaceActivity) {
        let activity: DbActivity = activity.to_owned().into();

        self.activities.insert(activity.id, activity);
    }

    pub fn fold_bidding(&mut self, activity: &NftMarketplaceActivity) {
        let result: Result<DbBid> = activity.to_owned().try_into();
        if let Ok(bid) = result {
            self.bids
                .entry(bid.id)
                .and_modify(|existing: &mut DbBid| {
                    if let Some(nonce) = bid.nonce.as_ref() {
                        existing.nonce = Some(nonce.to_string());
                    }

                    if let Some(tx_id) = bid.created_tx_id.as_ref() {
                        existing.created_tx_id = Some(tx_id.to_string());
                        existing.status = Some("active".to_string());
                    }

                    if let Some(tx_id) = bid.accepted_tx_id.as_ref() {
                        existing.accepted_tx_id = Some(tx_id.to_string());
                        existing.status = Some("matched".to_string());
                    }

                    if let Some(tx_id) = bid.cancelled_tx_id.as_ref() {
                        existing.cancelled_tx_id = Some(tx_id.to_string());
                        existing.status = Some("cancelled".to_string());
                    }

                    if let Some(receiver) = bid.receiver.as_ref() {
                        existing.receiver = Some(receiver.to_string());
                    }
                })
                .or_insert(bid);
        }
    }

    pub fn fold_listing(&mut self, activity: &NftMarketplaceActivity) {
        let result: Result<DbListing> = activity.to_owned().try_into();
        if let Ok(listing) = result {
            self.listings
                .entry(listing.id)
                .and_modify(|existing: &mut DbListing| {
                    let is_latest = listing
                        .block_time
                        .zip(existing.block_time)
                        .map_or(false, |(current, existing)| current.gt(&existing));

                    if is_latest {
                        existing.block_time = listing.block_time.clone();
                        existing.listed = listing.listed.clone();
                        existing.block_height = listing.block_height.clone();
                        existing.nft_id = listing.nft_id.clone();
                        existing.nonce = listing.nonce.clone();
                        existing.price = listing.price.clone();
                        existing.seller = listing.seller.clone();
                        existing.tx_index = listing.tx_index.clone();
                    }
                })
                .or_insert(listing);
        }
    }

    pub fn fold_collection(&mut self, activity: &NftMarketplaceActivity) {
        let result: Result<DbCollection> = activity.to_owned().try_into();
        if let Ok(collection) = result {
            self.collections.insert(collection.id, collection);
        }
    }

    pub fn fold_nfts(&mut self, activity: &NftMarketplaceActivity) {
        let result: Result<DbNft> = activity.to_owned().try_into();
        if let Ok(nft) = result {
            self.nfts.insert(nft.id, nft);
        }
    }

    pub fn drain(
        &mut self,
    ) -> (
        Vec<DbActivity>,
        Vec<DbBid>,
        Vec<DbListing>,
        Vec<DbCollection>,
        Vec<DbNft>,
    ) {
        (
            self.activities.drain().map(|(_, v)| v).collect(),
            self.bids.drain().map(|(_, v)| v).collect(),
            self.listings.drain().map(|(_, v)| v).collect(),
            self.collections.drain().map(|(_, v)| v).collect(),
            self.nfts.drain().map(|(_, v)| v).collect(),
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct NFTReductionStep<TDb: IDatabase, TCache: ICache>
where
    Self: Sized + Send + 'static,
{
    name: String,
    db: Arc<TDb>,
    cache: Arc<TCache>,
    accumulator: NFTAccumulator,
}

impl<TDb: IDatabase, TCache: ICache> NFTReductionStep<TDb, TCache> {
    pub fn new(name: &str, db: Arc<TDb>, cache: Arc<TCache>) -> Self {
        Self {
            name: name.to_string(),
            db,
            cache,
            accumulator: NFTAccumulator::default(),
        }
    }
}

#[async_trait::async_trait]
impl<TDb: IDatabase, TCache: ICache> Processable for NFTReductionStep<TDb, TCache>
where
    TDb: Send + Sync,
    TCache: 'static,
{
    type Input = (
        Vec<NftMarketplaceActivity>,
        HashMap<String, HashMap<String, String>>,
    );
    type Output = (
        Vec<DbActivity>,
        Vec<DbBid>,
        Vec<DbListing>,
        Vec<DbCollection>,
        Vec<DbNft>,
    );
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        input: TransactionContext<Self::Input>,
    ) -> Result<Option<TransactionContext<Self::Output>>, ProcessorError> {
        let (activities, resource_updates) = input.data;

        for activity in activities.iter() {
            let mut activity = activity.clone();

            let usd = match self.cache.get_token_price(APT_TOKEN_ADDR).await {
                Some(usd) => usd,
                None => self
                    .db
                    .token_prices()
                    .fetch_token_price(APT_TOKEN_ADDR)
                    .await
                    .unwrap_or_default(),
            };

            if let Some(token_addr) = activity.token_addr.as_ref() {
                if let Some(resource) = resource_updates.get(token_addr) {
                    for (column, value) in resource {
                        let field = MarketplaceField::from_str(column).unwrap();

                        if activity.get_field(field.clone()).is_none() {
                            activity.set_field(field.clone(), value.clone());
                        }
                    }
                }
            }

            activity.usd_price = Some(BigDecimal::from(activity.price) / APT_DECIMAL as i64 * &usd);

            self.accumulator.fold_activity(&activity);
            self.accumulator.fold_bidding(&activity);
            self.accumulator.fold_listing(&activity);
            self.accumulator.fold_collection(&activity);
            self.accumulator.fold_nfts(&activity);
        }

        let reduced_data = self.accumulator.drain();

        Ok(Some(TransactionContext {
            data: reduced_data,
            metadata: input.metadata,
        }))
    }
}

impl<TDb: IDatabase, TCache: ICache> AsyncStep for NFTReductionStep<TDb, TCache> {}

impl<TDb: IDatabase, TCache: ICache> NamedStep for NFTReductionStep<TDb, TCache> {
    fn name(&self) -> String {
        format!("{}NFTReductionStep", capitalize(&self.name))
    }
}
