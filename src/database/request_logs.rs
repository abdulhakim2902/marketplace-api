use anyhow::Context;
use chrono::{Datelike, TimeZone, Timelike, Utc};
use std::sync::Arc;
use uuid::Uuid;

use sqlx::{PgPool, postgres::PgQueryResult};

#[async_trait::async_trait]
pub trait IRequestLogs: Send + Sync {
    async fn add_logs(&self, api_key_id: &Uuid) -> anyhow::Result<PgQueryResult>;
}

pub struct RequestLogs {
    pool: Arc<PgPool>,
}

impl RequestLogs {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl IRequestLogs for RequestLogs {
    async fn add_logs(&self, api_key_id: &Uuid) -> anyhow::Result<PgQueryResult> {
        let now = Utc::now();
        let rounded = Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), now.hour(), 0, 0)
            .unwrap();
        let count = 1;

        let res = sqlx::query!(
            r#"
            INSERT INTO request_logs (api_key_id, ts, count)
            VALUES ($1, $2, $3)
            ON CONFLICT (api_key_id, ts) 
            DO UPDATE SET
              count = EXCLUDED.count + request_logs.count;
            "#,
            api_key_id,
            rounded,
            count,
        )
        .execute(&*self.pool)
        .await
        .context("Failed to fetch")?;

        Ok(res)
    }
}
