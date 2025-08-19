use anyhow::Context;
use chrono::{Datelike, TimeZone, Timelike, Utc};
use std::sync::Arc;
use uuid::Uuid;

use sqlx::{PgPool, postgres::PgQueryResult};

use crate::utils::generate_request_log_id;

#[async_trait::async_trait]
pub trait IRequestLogs: Send + Sync {
    async fn add_logs(&self, api_key_id: &Uuid, user_id: &Uuid) -> anyhow::Result<PgQueryResult>;
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
    async fn add_logs(&self, api_key_id: &Uuid, user_id: &Uuid) -> anyhow::Result<PgQueryResult> {
        let now = Utc::now();
        let rounded = Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), now.hour(), 0, 0)
            .unwrap();

        let count = 1;
        let ts = rounded.timestamp();
        let id = generate_request_log_id(&api_key_id.to_string(), ts);

        let res = sqlx::query!(
            r#"
            INSERT INTO request_logs (id, api_key_id, user_id, ts, count)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) 
            DO UPDATE SET
              count = EXCLUDED.count + request_logs.count;
            "#,
            id,
            api_key_id,
            user_id,
            rounded,
            count,
        )
        .execute(&*self.pool)
        .await
        .context("Failed to add logs")?;

        Ok(res)
    }
}
