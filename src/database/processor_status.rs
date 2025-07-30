use std::sync::Arc;

use crate::models::db::processor_status::DbProcessorStatus;
use anyhow::Context;
use sqlx::{PgPool, postgres::PgQueryResult};

#[async_trait::async_trait]
pub trait IProcessorStatus: Send + Sync {
    async fn get_starting_version(&self, processor_name: &str) -> anyhow::Result<i64>;
    async fn save_processor_status(
        &self,
        processor_status: &DbProcessorStatus,
    ) -> anyhow::Result<PgQueryResult>;
}

pub struct ProcessorStatus {
    pool: Arc<PgPool>,
}

impl ProcessorStatus {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl IProcessorStatus for ProcessorStatus {
    async fn get_starting_version(&self, processor_name: &str) -> anyhow::Result<i64> {
        let res = sqlx::query!(
            r#"
            SELECT last_success_version FROM processor_status WHERE processor = $1
            "#,
            processor_name
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch starting version")?;

        Ok(res.last_success_version)
    }

    async fn save_processor_status(
        &self,
        processor_status: &DbProcessorStatus,
    ) -> anyhow::Result<PgQueryResult> {
        let res = sqlx::query!(
            r#"
            INSERT INTO processor_status (processor, last_success_version, last_transaction_timestamp) 
            VALUES ($1, $2, $3)
            ON CONFLICT (processor) DO UPDATE
            SET last_success_version = $2, last_transaction_timestamp = $3
            "#,
            processor_status.processor,
            processor_status.last_success_version,
            processor_status.last_transaction_timestamp
        ).execute(&*self.pool)
        .await
        .context("Failed to save processor status")?;

        Ok(res)
    }
}
