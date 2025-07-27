use std::sync::Arc;

use aptos_indexer_processor_sdk::{
    aptos_indexer_transaction_stream::utils::time::parse_timestamp,
    common_steps::ProcessorStatusSaver, types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};

use crate::{
    database::{IDatabase, processor_status::IProcessorStatus},
    models::db::processor_status::ProcessorStatus,
};

pub struct DbProcessorStatusSaver<TDb: IDatabase> {
    pub name: String,
    pub db: Arc<TDb>,
}

impl<TDb: IDatabase> DbProcessorStatusSaver<TDb> {
    pub fn new(name: String, db: Arc<TDb>) -> Self {
        Self { name, db }
    }
}

#[async_trait::async_trait]
impl<TDb: IDatabase> ProcessorStatusSaver for DbProcessorStatusSaver<TDb> {
    async fn save_processor_status(
        &self,
        last_success_batch: &TransactionContext<()>,
    ) -> Result<(), ProcessorError> {
        let last_success_version = last_success_batch.metadata.end_version;
        let last_transaction_timestamp = last_success_batch
            .metadata
            .end_transaction_timestamp
            .as_ref()
            .map(|ts| parse_timestamp(ts, last_success_version as i64));

        let status = ProcessorStatus {
            processor: self.name.clone(),
            last_success_version: last_success_version as i64,
            last_transaction_timestamp,
        };

        self.db
            .processor_status()
            .save_processor_status(&status)
            .await
            .map_err(|e| ProcessorError::ProcessError {
                message: format!("Failed to save processor status: {e:#}"),
            })?;

        Ok(())
    }
}
