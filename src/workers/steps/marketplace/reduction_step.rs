use crate::{
    cache::ICache,
    database::{IDatabase, token_prices::ITokenPrices},
    models::{
        db::{activity::Activity, bid::Bid, listing::Listing},
        marketplace::{BidModel, ListingModel, NftMarketplaceActivity},
    },
};
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use bigdecimal::BigDecimal;
use std::{collections::HashMap, sync::Arc};

pub type BidIdType = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

pub type ListingIdType = (Option<String>, Option<String>);

const APT_TOKEN_ADDR: &str = "0x000000000000000000000000000000000000000000000000000000000000000a";
const APT_DECIMAL: i32 = 100_000_000;

#[derive(Clone, Debug, Default)]
pub struct NFTAccumulator {
    activities: HashMap<i64, Activity>,
    bids: HashMap<BidIdType, Bid>,
    listings: HashMap<ListingIdType, Listing>,
}

impl NFTAccumulator {
    pub fn fold_activity(&mut self, activity: &NftMarketplaceActivity) {
        let key = activity.get_tx_index();
        let activity: Activity = activity.to_owned().into();

        self.activities.insert(key, activity);
    }

    pub fn fold_bidding(&mut self, activity: &NftMarketplaceActivity) {
        if activity.is_valid_bid() {
            let bid: Bid = activity.to_owned().into();
            let key = (
                bid.market_contract_id.clone(),
                bid.collection_id.clone(),
                bid.nft_id.clone(),
                bid.bidder.clone(),
            );

            self.bids
                .entry(key)
                .and_modify(|existing: &mut Bid| {
                    if let Some(nonce) = bid.nonce.as_ref() {
                        existing.nonce = Some(nonce.to_string());
                    }

                    if let Some(tx_id) = bid.created_tx_id.as_ref() {
                        existing.created_tx_id = Some(tx_id.to_string());
                    }

                    if let Some(tx_id) = bid.accepted_tx_id.as_ref() {
                        existing.accepted_tx_id = Some(tx_id.to_string());
                        existing.status = Some("matched".to_string());
                    }

                    if let Some(tx_id) = bid.canceled_tx_id.as_ref() {
                        existing.canceled_tx_id = Some(tx_id.to_string());
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
            let listing: Listing = activity.to_owned().into();
            let key = (listing.market_contract_id.clone(), listing.nft_id.clone());
            self.listings
                .entry(key)
                .and_modify(|existing: &mut Listing| {
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
                        existing.price_str = listing.price_str.clone();
                        existing.seller = listing.seller.clone();
                        existing.tx_index = listing.tx_index.clone();

                        if !is_listed {
                            existing.nonce = None;
                            existing.price = None;
                            existing.price_str = None;
                            existing.seller = None;
                            existing.tx_index = None;
                        }
                    }
                })
                .or_insert(listing);
        }
    }

    pub fn drain(&mut self) -> (Vec<Activity>, Vec<Bid>, Vec<Listing>) {
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
    type Input = Vec<Vec<NftMarketplaceActivity>>;
    type Output = (Vec<Activity>, Vec<Bid>, Vec<Listing>);
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        input: TransactionContext<Self::Input>,
    ) -> Result<Option<TransactionContext<Self::Output>>, ProcessorError> {
        for activities in input.data.iter() {
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

                let usd_price =
                    BigDecimal::from(activity.price) * &usd / BigDecimal::from(APT_DECIMAL);

                activity.usd_price = Some(usd_price);

                self.accumulator.fold_activity(&activity);
                self.accumulator.fold_bidding(&activity);
                self.accumulator.fold_listing(&activity);
            }
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
