use std::sync::Arc;

use crate::models::{
    db::activity::DbActivity,
    schema::{
        activity::{
            ActivitySchema, WhereActivitySchema,
            profit_loss::{ProfitLossSchema, WhereProfitLossSchema},
        },
        collection::CollectionSaleSchema,
        data_point::DataPointSchema,
    },
};
use anyhow::Context;
use bigdecimal::BigDecimal;
use sqlx::{
    PgPool, Postgres, QueryBuilder, Transaction,
    postgres::{PgQueryResult, types::PgInterval},
};

#[async_trait::async_trait]
pub trait IActivities: Send + Sync {
    async fn tx_insert_activities(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbActivity>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_activities(
        &self,
        query: &WhereActivitySchema,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ActivitySchema>>;

    async fn fetch_past_floor(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Option<BigDecimal>>;

    async fn fetch_sale(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<CollectionSaleSchema>;

    async fn fetch_contribution_chart(
        &self,
        wallet_address: &str,
    ) -> anyhow::Result<Vec<DataPointSchema>>;

    async fn fetch_profit_and_loss(
        &self,
        query: &WhereProfitLossSchema,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ProfitLossSchema>>;
}

pub struct Activities {
    pool: Arc<PgPool>,
}

impl Activities {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl IActivities for Activities {
    async fn tx_insert_activities(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbActivity>,
    ) -> anyhow::Result<PgQueryResult> {
        if items.is_empty() {
            return Ok(PgQueryResult::default());
        }

        let res = QueryBuilder::<Postgres>::new(
            r#"
            INSERT INTO activities (
                id,
                tx_type,
                tx_index,
                tx_id,
                sender,
                receiver,
                price,
                nft_id,
                collection_id,
                market_contract_id,
                market_name,
                usd_price,
                block_time,
                block_height,
                amount
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.id);
            b.push_bind(item.tx_type);
            b.push_bind(item.tx_index);
            b.push_bind(item.tx_id.clone());
            b.push_bind(item.sender.clone());
            b.push_bind(item.receiver.clone());
            b.push_bind(item.price);
            b.push_bind(item.nft_id.clone());
            b.push_bind(item.collection_id.clone());
            b.push_bind(item.market_contract_id.clone());
            b.push_bind(item.market_name.clone());
            b.push_bind(item.usd_price);
            b.push_bind(item.block_time);
            b.push_bind(item.block_height);
            b.push_bind(item.amount);
        })
        .push(
            r#"
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert activities")?;

        Ok(res)
    }

    async fn fetch_activities(
        &self,
        query: &WhereActivitySchema,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ActivitySchema>> {
        let res = sqlx::query_as!(
            ActivitySchema,
            r#"
            SELECT 
                a.tx_type,
                a.tx_index,
                a.tx_id,
                a.sender,
                a.receiver,
                a.price,
                a.usd_price,
                a.market_name,
                a.market_contract_id,
                a.nft_id,
                a.collection_id,
                a.block_time,
                a.block_height,
                a.amount
            FROM activities a
            WHERE (($1::TEXT IS NULL OR $1::TEXT = '' OR a.sender = $1) OR ($1::TEXT IS NULL OR $1::TEXT = '' OR a.receiver = $1))
                AND ($2::TEXT IS NULL OR $2::TEXT = '' OR a.collection_id = $2)
                AND ($3::TEXT IS NULL OR $3::TEXT = '' OR a.nft_id = $3)
            ORDER BY a.block_time
            LIMIT $4 OFFSET $5
            "#,
            query.wallet_address,
            query.collection_id,
            query.nft_id,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection activities")?;

        Ok(res)
    }

    async fn fetch_past_floor(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Option<BigDecimal>> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT SUM(a.price) FROM activities a
            WHERE a.tx_type = 'buy' 
                AND a.collection_id = $1
                AND ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL)
            GROUP BY a.collection_id
            "#,
            collection_id,
            interval,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch volume")?;

        Ok(res)
    }

    async fn fetch_sale(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<CollectionSaleSchema> {
        let res = sqlx::query_as!(
            CollectionSaleSchema,
            r#"
            SELECT COUNT(*) AS total, SUM(a.price) AS volume, SUM(a.usd_price) AS volume_usd
            FROM activities a
            WHERE a.tx_type = 'buy'
                AND a.collection_id = $1
                AND ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL)
            GROUP BY a.collection_id
            "#,
            collection_id,
            interval,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch sales")?;

        Ok(res)
    }

    async fn fetch_contribution_chart(
        &self,
        wallet_address: &str,
    ) -> anyhow::Result<Vec<DataPointSchema>> {
        let res = sqlx::query_as!(
            DataPointSchema,
            r#"
            WITH 
                time_series AS (
                    SELECT generate_series(DATE_TRUNC('day', NOW() - '1 year'::INTERVAL), DATE_TRUNC('day', NOW()), '1d'::INTERVAL) AS time_bin
                ),
                activity_counts AS (
                    SELECT 
                        DATE_TRUNC('day', a.block_time) AS block_time_trunc, 
                        COUNT(*) FROM activities a
                    WHERE a.receiver = $1 OR a.sender = $1
                    GROUP BY block_time_trunc
                )
            SELECT
                ts.time_bin AS x, 
                COALESCE(ac.count, 0) AS y 
            FROM time_series ts
                LEFT JOIN activity_counts ac ON ts.time_bin = ac.block_time_trunc
            ORDER BY ts.time_bin
            "#,
            wallet_address,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch activity contributons chart")?;

        Ok(res)
    }

    async fn fetch_profit_and_loss(
        &self,
        query: &WhereProfitLossSchema,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ProfitLossSchema>> {
        let res = sqlx::query_as!(
            ProfitLossSchema,
            r#"
            SELECT
                ra.collection_id,
                ra.nft_id,
                ra.price            AS bought,
                ra.usd_price        AS bought_usd,
                sa.price            AS sold,
                sa.usd_price        AS sold_usd
            FROM activities ra
                LEFT JOIN activities sa ON ra.receiver = sa.sender AND ra.nft_id = sa.nft_id
            WHERE ra.receiver IS NOT NULL
                AND ($1::TEXT IS NULL OR $1::TEXT = '' OR ra.receiver = $1)
            LIMIT $2 OFFSET $3
            "#,
            query.wallet_address,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch activity contributons chart")?;

        Ok(res)
    }
}
