use crate::{
    cache::ICache,
    database::{IDatabase, token_prices::ITokenPrices},
    models::{
        db::{activity::DbActivity, bid::DbBid, listing::DbListing},
        marketplace::{
            APT_DECIMAL, BidModel, ListingModel, MarketplaceField, MarketplaceModel,
            NftMarketplaceActivity,
        },
    },
};
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use bigdecimal::BigDecimal;
use std::{collections::HashMap, str::FromStr, sync::Arc};

const APT_TOKEN_ADDR: &str = "0x000000000000000000000000000000000000000000000000000000000000000a";

#[derive(Clone, Debug, Default)]
pub struct NFTAccumulator {
    activities: HashMap<String, DbActivity>,
    bids: HashMap<String, DbBid>,
    listings: HashMap<String, DbListing>,
}

impl NFTAccumulator {
    pub fn fold_activity(&mut self, activity: &NftMarketplaceActivity) {
        let key = activity.get_activity_id();
        let activity: DbActivity = activity.to_owned().into();

        self.activities.insert(key, activity);
    }

    pub fn fold_bidding(&mut self, activity: &NftMarketplaceActivity) {
        if activity.is_valid_bid() {
            let bid: DbBid = activity.to_owned().into();
            let key = bid.id.as_ref().unwrap();

            self.bids
                .entry(key.to_string())
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
        if activity.is_valid_listing() {
            let listing: DbListing = activity.to_owned().into();
            let key = listing.id.as_ref().unwrap();
            self.listings
                .entry(key.to_string())
                .and_modify(|existing: &mut DbListing| {
                    let is_listed = listing.listed.unwrap_or(false);
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

                        if !is_listed {
                            existing.nonce = None;
                            existing.price = None;
                            existing.seller = None;
                            existing.tx_index = None;
                        }
                    }
                })
                .or_insert(listing);
        }
    }

    pub fn drain(&mut self) -> (Vec<DbActivity>, Vec<DbBid>, Vec<DbListing>) {
        (
            self.activities.drain().map(|(_, v)| v).collect(),
            self.bids.drain().map(|(_, v)| v).collect(),
            self.listings.drain().map(|(_, v)| v).collect(),
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct NFTReductionStep<TDb: IDatabase, TCache: ICache>
where
    Self: Sized + Send + 'static,
{
    db: Arc<TDb>,
    cache: Arc<TCache>,
    accumulator: NFTAccumulator,
}

impl<TDb: IDatabase, TCache: ICache> NFTReductionStep<TDb, TCache> {
    pub fn new(db: Arc<TDb>, cache: Arc<TCache>) -> Self {
        Self {
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
    type Output = (Vec<DbActivity>, Vec<DbBid>, Vec<DbListing>);
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
                    .get_token_price(APT_TOKEN_ADDR)
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
        "NFTReductionStep".to_string()
    }
}
