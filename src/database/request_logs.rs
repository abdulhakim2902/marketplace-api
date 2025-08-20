use crate::models::api::responses::log::UserLogSummaryResponse;
use anyhow::Context;
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;

use sqlx::{
    PgPool,
    postgres::{PgQueryResult, types::PgInterval},
};

use crate::{models::schema::data_point::DataPointSchema, utils::generate_request_log_id};

#[async_trait::async_trait]
pub trait IRequestLogs: Send + Sync {
    async fn add_logs(&self, api_key_id: &Uuid, user_id: &Uuid) -> anyhow::Result<PgQueryResult>;

    async fn fetch_logs(
        &self,
        user_id: &str,
        api_key_id: Option<&str>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        interval: PgInterval,
    ) -> anyhow::Result<Vec<DataPointSchema>>;

    async fn fetch_summaries(
        &self,
        user_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> anyhow::Result<Vec<UserLogSummaryResponse>>;
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
            .with_ymd_and_hms(
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                0,
            )
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

    async fn fetch_logs(
        &self,
        user_id: &str,
        api_key_id: Option<&str>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        interval: PgInterval,
    ) -> anyhow::Result<Vec<DataPointSchema>> {
        let res = sqlx::query_as!(
            DataPointSchema,
            r#"
            WITH 
                time_series AS (
                    SELECT GENERATE_SERIES($3::TIMESTAMPTZ, $4::TIMESTAMPTZ, $5::INTERVAL) AS time_bin
                ),
                user_logs AS (
                    SELECT rl.ts, SUM(rl.count) AS count FROM request_logs rl
                    WHERE rl.user_id = $1 
                        AND ($2::UUID IS NULL OR rl.api_key_id = $2)
                    GROUP BY ts
                )
            SELECT 
                ts.time_bin                         AS x, 
                COALESCE(SUM(ul.count), 0)::BIGINT  AS y
            FROM time_series ts
                LEFT JOIN user_logs ul ON ul.ts >= ts.time_bin AND ul.ts < ts.time_bin + $5::INTERVAL
            GROUP BY ts.time_bin
            ORDER BY ts.time_bin
            "#,
            Uuid::from_str(user_id).ok(),
            api_key_id.map(|id| Uuid::from_str(id).ok()).flatten(),
            start_time,
            end_time,
            interval,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch user logs")?;

        Ok(res)
    }

    async fn fetch_summaries(
        &self,
        user_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> anyhow::Result<Vec<UserLogSummaryResponse>> {
        let res = sqlx::query_as!(
            UserLogSummaryResponse,
            r#"
            SELECT
                rl.api_key_id,
                SUM(rl.count)::BIGINT     AS total 
            FROM request_logs rl
            WHERE rl.user_id = $1
                AND rl.ts BETWEEN $2 AND $3
            GROUP BY rl.user_id, rl.api_key_id
            "#,
            Uuid::from_str(user_id).ok(),
            start_time,
            end_time,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch user log summaries")?;

        Ok(res)
    }
}
