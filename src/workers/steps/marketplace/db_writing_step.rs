use std::sync::Arc;

use crate::{
    database::{
        IDatabase, activities::IActivities, bids::IBids, collections::ICollections,
        listings::IListings, nfts::INfts,
    },
    models::db::{
        activity::DbActivity, bid::DbBid, collection::DbCollection, listing::DbListing, nft::DbNft,
    },
    utils::string_utils::capitalize,
};
use aptos_indexer_processor_sdk::{
    traits::{AsyncStep, NamedStep, Processable, async_step::AsyncRunType},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};

pub struct DBWritingStep<TDb: IDatabase> {
    pub name: String,
    pub db: Arc<TDb>,
}

impl<TDb: IDatabase> DBWritingStep<TDb> {
    pub fn new(name: &str, db: Arc<TDb>) -> Self {
        Self {
            name: name.to_string(),
            db,
        }
    }
}

#[async_trait::async_trait]
impl<TDb: IDatabase> Processable for DBWritingStep<TDb>
where
    TDb: IDatabase + Send + Sync,
{
    type Input = (
        Vec<DbActivity>,
        Vec<DbBid>,
        Vec<DbListing>,
        Vec<DbCollection>,
        Vec<DbNft>,
    );
    type Output = ();
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        input: TransactionContext<Self::Input>,
    ) -> Result<Option<TransactionContext<()>>, ProcessorError> {
        let (activities, bids, listings, collections, nfts) = input.data;

        let mut tx =
            self.db
                .get_pool()
                .begin()
                .await
                .map_err(|e| ProcessorError::ProcessError {
                    message: format!("{e:#}"),
                })?;

        self.db
            .activities()
            .tx_insert_activities(&mut tx, activities)
            .await
            .map_err(|e| ProcessorError::ProcessError {
                message: format!("{e:#}"),
            })?;

        self.db
            .bids()
            .tx_insert_bids(&mut tx, bids)
            .await
            .map_err(|e| ProcessorError::ProcessError {
                message: format!("{e:#}"),
            })?;

        self.db
            .listings()
            .tx_insert_listings(&mut tx, listings)
            .await
            .map_err(|e| ProcessorError::ProcessError {
                message: format!("{e:#}"),
            })?;

        self.db
            .collections()
            .tx_insert_collections(&mut tx, collections)
            .await
            .map_err(|e| ProcessorError::ProcessError {
                message: format!("{e:#}"),
            })?;

        self.db
            .nfts()
            .tx_insert_nfts(&mut tx, nfts)
            .await
            .map_err(|e| ProcessorError::ProcessError {
                message: format!("{e:#}"),
            })?;

        tx.commit()
            .await
            .map_err(|e| ProcessorError::ProcessError {
                message: format!("Failed to commit transaction: {e:#}"),
            })?;

        Ok(Some(TransactionContext {
            data: (),
            metadata: input.metadata,
        }))
    }
}

impl<TDb: IDatabase> AsyncStep for DBWritingStep<TDb> {}

impl<TDb: IDatabase> NamedStep for DBWritingStep<TDb> {
    fn name(&self) -> String {
        format!("{}DBWritingStep", capitalize(&self.name))
    }
}
