use std::{collections::HashMap, sync::Arc};

use crate::{
    database::Schema,
    models::{
        db::activity::DbActivity,
        schema::{
            AggregateFieldsSchema,
            activity::{
                ActivitySchema, AggregateActivityFieldsSchema, DistinctActivitySchema,
                OrderActivitySchema, QueryActivitySchema, profit_loss::ProfitLossSchema,
            },
            data_point::DataPointSchema,
        },
    },
    utils::schema::{create_aggregate_query_builder, create_query_builder},
};
use anyhow::Context;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};

#[async_trait::async_trait]
pub trait IActivities: Send + Sync {
    async fn tx_insert_activities(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbActivity>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_activities(
        &self,
        query: &QueryActivitySchema,
        order: &OrderActivitySchema,
        distinct: Option<&DistinctActivitySchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ActivitySchema>>;

    async fn fetch_aggregate_activities(
        &self,
        selection: &HashMap<String, Vec<String>>,
        query: &QueryActivitySchema,
        distinct: Option<&DistinctActivitySchema>,
    ) -> anyhow::Result<AggregateFieldsSchema<AggregateActivityFieldsSchema>>;

    async fn fetch_contribution_chart(
        &self,
        wallet_address: &str,
    ) -> anyhow::Result<Vec<DataPointSchema>>;

    async fn fetch_profit_and_loss(
        &self,
        wallet_address: &str,
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
        query: &QueryActivitySchema,
        order: &OrderActivitySchema,
        distinct: Option<&DistinctActivitySchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ActivitySchema>> {
        create_query_builder(
            "activities",
            Schema::Activities,
            query,
            order,
            distinct,
            limit,
            offset,
        )
        .build_query_as::<ActivitySchema>()
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch activities")
    }

    async fn fetch_aggregate_activities(
        &self,
        selection: &HashMap<String, Vec<String>>,
        query: &QueryActivitySchema,
        distinct: Option<&DistinctActivitySchema>,
    ) -> anyhow::Result<AggregateFieldsSchema<AggregateActivityFieldsSchema>> {
        if selection.is_empty() {
            return Ok(AggregateFieldsSchema::default());
        }

        let table = if let Some(distinct) = distinct {
            format!("(SELECT DISTINCT ON ({}) * FROM activities)", distinct)
        } else {
            "(SELECT * FROM activities)".to_string()
        };

        let value =
            create_aggregate_query_builder(table.as_str(), selection, Schema::Activities, query)
                .build_query_scalar::<serde_json::Value>()
                .fetch_one(&*self.pool)
                .await
                .context("Failed to fetch aggregate activities")?;

        let result =
            serde_json::from_value::<AggregateFieldsSchema<AggregateActivityFieldsSchema>>(value)
                .context("Failed to parse aggregate result")?;

        Ok(result)
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
        wallet_address: &str,
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
            wallet_address,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch activity contributons chart")?;

        Ok(res)
    }
}
