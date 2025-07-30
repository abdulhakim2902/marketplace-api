use std::sync::Arc;

use crate::{
    database::{IDatabase, activities::IActivities, bids::IBids, listings::IListings},
    models::db::{activity::DbActivity, bid::DbBid, listing::DbListing},
};
use aptos_indexer_processor_sdk::{
    traits::{AsyncStep, NamedStep, Processable, async_step::AsyncRunType},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};

pub struct DBWritingStep<TDb: IDatabase> {
    pub db: Arc<TDb>,
}

impl<TDb: IDatabase> DBWritingStep<TDb> {
    pub fn new(db: Arc<TDb>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl<TDb: IDatabase> Processable for DBWritingStep<TDb>
where
    TDb: IDatabase + Send + Sync,
{
    type Input = (Vec<DbActivity>, Vec<DbBid>, Vec<DbListing>);
    type Output = ();
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        input: TransactionContext<Self::Input>,
    ) -> Result<Option<TransactionContext<()>>, ProcessorError> {
        let (activities, bids, listings) = input.data;

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
        "DBMarketplaceWritingStep".to_string()
    }
}
