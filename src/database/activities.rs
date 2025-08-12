use std::{str::FromStr, sync::Arc};

use crate::models::schema::activity::FilterActivitySchema;
use crate::models::schema::activity::profit_loss::FilterProfitLossSchema;
use crate::models::{
    db::activity::DbActivity,
    schema::{
        activity::{ActivitySchema, TxType, profit_loss::ProfitLossSchema},
        data_point::DataPointSchema,
    },
};
use anyhow::Context;
use sqlx::{
    PgPool, Postgres, QueryBuilder, Transaction,
    postgres::{PgQueryResult, types::PgInterval},
};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait IActivities: Send + Sync {
    async fn tx_insert_activities(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbActivity>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_activities(
        &self,
        filter: FilterActivitySchema,
    ) -> anyhow::Result<Vec<ActivitySchema>>;

    async fn fetch_past_floor(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<i64>;

    async fn fetch_contribution_chart(
        &self,
        wallet_address: &str,
    ) -> anyhow::Result<Vec<DataPointSchema>>;

    async fn fetch_profit_and_loss(
        &self,
        filter: Option<FilterProfitLossSchema>,
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
        filter: FilterActivitySchema,
    ) -> anyhow::Result<Vec<ActivitySchema>> {
        let query = filter.where_.unwrap_or_default();
        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                a.id,
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
            WHERE TRUE
            "#,
        );

        if let Some(id) = query.id.as_ref() {
            let activity_id = Uuid::from_str(id).ok();
            query_builder.push(" AND a.id = ");
            query_builder.push_bind(activity_id);
        }

        if let Some(addr) = query.wallet_address.as_ref() {
            query_builder.push(" AND (a.sender = ");
            query_builder.push_bind(addr);
            query_builder.push(" OR a.receiver = ");
            query_builder.push_bind(addr);
            query_builder.push(")");
        }

        if let Some(collection_id) = query.collection_id.as_ref() {
            query_builder.push(" AND a.collection_id = ");
            query_builder.push_bind(collection_id);
        }

        if let Some(nft_id) = query.nft_id.as_ref() {
            let nft_id = Uuid::from_str(nft_id).ok();

            query_builder.push(" AND a.nft_id = ");
            query_builder.push_bind(nft_id);
        }

        if let Some(tx_types) = query.tx_types.as_ref() {
            let mut types = Vec::new();

            for tx_type in tx_types {
                match tx_type {
                    TxType::Mints => {
                        types.push("mint".to_string());
                    }
                    TxType::Transfers => {
                        types.push("transfer".to_string());
                    }
                    TxType::Sales => {
                        types.push("buy".to_string());
                    }
                    TxType::Offers => {
                        types.push("solo-bid".to_string());
                        types.push("unlist-bid".to_string());
                        types.push("accept-bid".to_string());
                        types.push("collection-bid".to_string());
                        types.push("cancel-collection-bid".to_string());
                        types.push("accept-collection-bid".to_string());
                    }
                    TxType::Listings => {
                        types.push("list".to_string());
                        types.push("unlist".to_string());
                    }
                }
            }

            query_builder.push(" AND a.tx_type = ANY(");
            query_builder.push_bind(types);
            query_builder.push(")");
        }

        query_builder.push(" ORDER BY a.block_time, a.tx_index ASC");
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let res = query_builder
            .build_query_as::<ActivitySchema>()
            .fetch_all(&*self.pool)
            .await
            .context("Failed to fetch activities")?;

        Ok(res)
    }

    async fn fetch_past_floor(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT MIN(a.price) FROM activities a
            WHERE a.tx_type = 'list' 
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

        Ok(res.unwrap_or_default())
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
        filter: Option<FilterProfitLossSchema>,
    ) -> anyhow::Result<Vec<ProfitLossSchema>> {
        let filter = filter.unwrap_or_default();

        let query = filter.where_.unwrap_or_default();
        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

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
