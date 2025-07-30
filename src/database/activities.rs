use std::sync::Arc;

use crate::models::{
    api::responses::{
        activity::Activity, collection::CollectionSale, data_point::DataPoint,
        nft_change::NftChange, profit_leaderboard::ProfitLeaderboard, top_buyer::TopBuyer,
        top_seller::TopSeller,
    },
    db::activity::Activity as DbActivity,
};
use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
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

    async fn fetch_activities(&self, limit: i64, offset: i64) -> anyhow::Result<Vec<Activity>>;

    async fn fetch_past_floor(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Option<BigDecimal>>;

    async fn fetch_sale(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<CollectionSale>;

    async fn fetch_floor_chart(
        &self,
        collection_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        interval: PgInterval,
    ) -> anyhow::Result<Vec<DataPoint>>;

    async fn fetch_top_buyers(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<TopBuyer>>;

    async fn fetch_top_sellers(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<TopSeller>>;

    async fn fetch_profit_leaderboard(
        &self,
        collection_id: &str,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ProfitLeaderboard>>;

    async fn fetch_nft_changes(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<NftChange>>;
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
            ON CONFLICT (tx_index, tx_id) DO NOTHING
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert activities")?;

        Ok(res)
    }

    async fn fetch_activities(&self, limit: i64, offset: i64) -> anyhow::Result<Vec<Activity>> {
        let res = sqlx::query_as!(
            Activity,
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
            ORDER BY a.block_time
            LIMIT $1 OFFSET $2
            "#,
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
    ) -> anyhow::Result<CollectionSale> {
        let res = sqlx::query_as!(
            CollectionSale,
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

    async fn fetch_floor_chart(
        &self,
        collection_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        interval: PgInterval,
    ) -> anyhow::Result<Vec<DataPoint>> {
        let res = sqlx::query_as!(
            DataPoint,
            r#"
            WITH 
                time_series AS (
                    SELECT GENERATE_SERIES($2::TIMESTAMPTZ, $3::TIMESTAMPTZ, $4::INTERVAL) AS time_bin
                ),
                floor_prices AS (
                    SELECT 
                        ts.time_bin AS time,
                        COALESCE(
                            (
                                SELECT a.price FROM activities a
                                WHERE a.tx_type = 'list'
                                    AND a.collection_id = $1
                                    AND a.block_time >= ts.time_bin AND a.block_time < ts.time_bin + $4::INTERVAL
                                ORDER BY a.price ASC
                                LIMIT 1
                            ),
                            0
                        ) AS floor
                    FROM time_series ts
                    ORDER BY ts.time_bin
                )
            SELECT 
                ts.time_bin AS x,
                COALESCE(
                    (
                        SELECT fp.floor FROM floor_prices fp
                        WHERE fp.time <= ts.time_bin
                        LIMIT 1
                    ),
                    (
                        SELECT a.price FROM activities a
                        WHERE a.tx_type = 'list'
                            AND a.collection_id = $1
                            AND a.block_time <= ts.time_bin
                        ORDER BY a.price ASC
                        LIMIT 1
                    ),
                    0
                ) AS y
            FROM time_series ts
            ORDER BY ts.time_bin
            "#,
            collection_id,
            start_time,
            end_time,
            interval,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection floor chart")?;

        Ok(res)
    }

    async fn fetch_top_buyers(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<TopBuyer>> {
        let res = sqlx::query_as!(
            TopBuyer,
            r#"
            SELECT
                a.receiver      AS buyer, 
                COUNT(*)        AS bought, 
                SUM(a.price)    AS volume
            FROM activities a
            WHERE a.tx_type = 'buy'
                AND a.collection_id = $1
                AND ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL)
            GROUP BY a.collection_id, a.receiver
            ORDER BY bought DESC, volume DESC
            LIMIT 10
            "#,
            collection_id,
            interval,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection top buyers")?;

        Ok(res)
    }

    async fn fetch_top_sellers(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<TopSeller>> {
        let res = sqlx::query_as!(
            TopSeller,
            r#"
            SELECT
                a.sender            AS seller, 
                COUNT(*)            AS sold, 
                SUM(a.price)        AS volume
            FROM activities a
            WHERE a.tx_type = 'buy'
                AND a.collection_id = $1
                AND ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL)
            GROUP BY a.collection_id, a.sender
            ORDER BY sold DESC, volume DESC
            LIMIT 10
            "#,
            collection_id,
            interval,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection top buyers")?;

        Ok(res)
    }

    async fn fetch_profit_leaderboard(
        &self,
        collection_id: &str,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ProfitLeaderboard>> {
        let res = sqlx::query_as!(
            ProfitLeaderboard,
            r#"
            WITH
                bought_activities AS (
                    SELECT a.collection_id, a.receiver AS address, COUNT(*) AS bought, SUM(price) AS price FROM activities a
                    WHERE a.tx_type = 'buy' AND a.collection_id = $1
                    GROUP BY a.collection_id, a.receiver 
                ),
                sold_activities AS (
                    SELECT a.collection_id, a.sender AS address, COUNT(*) AS sold, SUM(price) AS price FROM activities a
                    WHERE a.tx_type = 'buy' AND a.collection_id = $1
                    GROUP BY a.collection_id, a.sender
                ),
                unique_addresses AS (
                    SELECT ba.address FROM bought_activities ba
                    UNION
                    SELECT sa.address FROM sold_activities sa
                )
            SELECT
                ua.address,
                ba.bought, 
                sa.sold, 
                ba.price                                                                AS spent,
                (COALESCE(sa.price, 0) - COALESCE(ba.price, 0)) 	                    AS total_profit
            FROM unique_addresses ua
                LEFT JOIN bought_activities ba ON ba.address = ua.address
                LEFT JOIN sold_activities sa ON sa.address = ua.address
            WHERE ua.address IS NOT NULL
            LIMIT $2 OFFSET $3
            "#,
            collection_id,
            limit,
            offset,
        ).fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection profit leaders")?;

        Ok(res)
    }

    async fn fetch_nft_changes(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<NftChange>> {
        let res = sqlx::query_as!(
            NftChange,
            r#"
            WITH 
                current_nft_owners AS (
                    SELECT n.owner, COUNT(*) FROM nfts n
                    WHERE n.burned IS NULL OR NOT n.burned AND n.collection_id = $1
                    GROUP BY n.collection_id, n.owner
                ),
                transfer_in AS (
                    SELECT a.collection_id, a.receiver AS address, COUNT(*) FROM activities a
                    WHERE ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL) 
                        AND a.tx_type IN ('transfer', 'buy')
                        AND a.collection_id = $1
                    GROUP BY a.collection_id, a.receiver
                ),
                transfer_out AS (
                    SELECT a.collection_id, a.sender AS address, COUNT(*) FROM activities a
                    WHERE ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL) 
                        AND a.tx_type IN ('transfer', 'buy')
                        AND a.collection_id = $1
                    GROUP BY a.collection_id, a.sender
                ),
                unique_addresses AS (
                    SELECT tin.address FROM transfer_in tin
                    UNION
                    SELECT tout.address FROM transfer_out tout
                )
            SELECT 
                ua.address, 
                (COALESCE(tout.count, 0) - COALESCE(tin.count, 0)) 	AS change,
                COALESCE(co.count, 0) 								AS quantity	
            FROM unique_addresses ua
                LEFT JOIN transfer_in tin ON tin.address = ua.address
                LEFT JOIN transfer_out tout ON tout.address = ua.address
                LEFT JOIN current_nft_owners co ON co.owner = ua.address
            WHERE ua.address IS NOT NULL
            ORDER BY change DESC
            LIMIT $3 OFFSET $4
            "#,
            collection_id,
            interval,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection profit leaders")?;

        Ok(res)
    }
}
