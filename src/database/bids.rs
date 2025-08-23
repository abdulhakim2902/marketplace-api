use std::{collections::HashMap, sync::Arc};

use crate::models::schema::AggregateFieldsSchema;
use crate::models::schema::bid::{AggregateBidFieldsSchema, DistinctBidSchema};
use crate::utils::schema::{create_aggregate_query_builder, create_query_builder};
use crate::{
    database::Schema,
    models::{
        db::bid::DbBid,
        schema::bid::{BidSchema, OrderBidSchema, QueryBidSchema},
    },
};
use anyhow::Context;
use chrono::Utc;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};

#[async_trait::async_trait]
pub trait IBids: Send + Sync {
    async fn tx_insert_bids(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        bids: Vec<DbBid>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_bids(
        &self,
        query: &QueryBidSchema,
        order: &OrderBidSchema,
        distinct: Option<&DistinctBidSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<BidSchema>>;

    async fn fetch_aggregate_bids(
        &self,
        selection: &HashMap<String, Vec<String>>,
        query: &QueryBidSchema,
        distinct: Option<&DistinctBidSchema>,
    ) -> anyhow::Result<AggregateFieldsSchema<AggregateBidFieldsSchema>>;
}

pub struct Bids {
    pool: Arc<PgPool>,
}

impl Bids {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl IBids for Bids {
    async fn tx_insert_bids(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbBid>,
    ) -> anyhow::Result<PgQueryResult> {
        if items.is_empty() {
            return Ok(PgQueryResult::default());
        }

        let res = QueryBuilder::<Postgres>::new(
            r#"
            INSERT INTO bids (
                id,
                bidder, 
                accepted_tx_id, 
                cancelled_tx_id, 
                collection_id, 
                created_tx_id, 
                expired_at, 
                market_contract_id, 
                market_name, 
                nonce, 
                nft_id, 
                price,
                receiver, 
                remaining_count, 
                status,
                bid_type,
                updated_at
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.id.clone());
            b.push_bind(item.bidder.clone());
            b.push_bind(item.accepted_tx_id.clone());
            b.push_bind(item.cancelled_tx_id.clone());
            b.push_bind(item.collection_id.clone());
            b.push_bind(item.created_tx_id.clone());
            b.push_bind(item.expired_at);
            b.push_bind(item.market_contract_id.clone());
            b.push_bind(item.market_name.clone());
            b.push_bind(item.nonce.clone());
            b.push_bind(item.nft_id.clone());
            b.push_bind(item.price);
            b.push_bind(item.receiver.clone());
            b.push_bind(item.remaining_count);
            b.push_bind(item.status.clone());
            b.push_bind(item.bid_type.clone());
            b.push_bind(Utc::now());
        })
        .push(
            r#"
            ON CONFLICT (id) DO UPDATE
            SET 
                bidder = EXCLUDED.bidder,
                status = EXCLUDED.status,
                nonce = EXCLUDED.nonce,
                created_tx_id = COALESCE(EXCLUDED.created_tx_id, bids.created_tx_id),
                accepted_tx_id = COALESCE(EXCLUDED.accepted_tx_id, bids.accepted_tx_id),
                cancelled_tx_id = COALESCE(EXCLUDED.cancelled_tx_id, bids.cancelled_tx_id),
                nft_id = COALESCE(EXCLUDED.nft_id, bids.nft_id),
                receiver = EXCLUDED.receiver,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert bids")?;

        Ok(res)
    }

    async fn fetch_bids(
        &self,
        query: &QueryBidSchema,
        order: &OrderBidSchema,
        distinct: Option<&DistinctBidSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<BidSchema>> {
        create_query_builder("bids", Schema::Bids, query, order, distinct, limit, offset)
            .build_query_as::<BidSchema>()
            .fetch_all(&*self.pool)
            .await
            .context("Failed to fetch bids")
    }

    async fn fetch_aggregate_bids(
        &self,
        selection: &HashMap<String, Vec<String>>,
        query: &QueryBidSchema,
        distinct: Option<&DistinctBidSchema>,
    ) -> anyhow::Result<AggregateFieldsSchema<AggregateBidFieldsSchema>> {
        if selection.is_empty() {
            return Ok(AggregateFieldsSchema::default());
        }

        let table = if let Some(distinct) = distinct {
            format!("(SELECT DISTINCT ON ({}) * FROM bids)", distinct)
        } else {
            "(SELECT * FROM bids)".to_string()
        };

        let value = create_aggregate_query_builder(table.as_str(), selection, Schema::Bids, query)
            .build_query_scalar::<serde_json::Value>()
            .fetch_one(&*self.pool)
            .await
            .context("Failed to fetch aggregate bids")?;

        let result: AggregateFieldsSchema<AggregateBidFieldsSchema> =
            serde_json::from_value::<AggregateFieldsSchema<AggregateBidFieldsSchema>>(value)
                .context("Failed to parse aggregate result")?;

        Ok(result)
    }
}
