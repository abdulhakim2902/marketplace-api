use std::{str::FromStr, sync::Arc};

use crate::models::schema::bid::DistinctBidSchema;
use crate::utils::schema::{create_count_query_builder, create_query_builder};
use crate::{
    database::Schema,
    models::{
        db::bid::DbBid,
        schema::bid::{BidSchema, OrderBidSchema, QueryBidSchema},
    },
};
use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::Utc;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};
use uuid::Uuid;

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

    async fn fetch_total_bids(
        &self,
        query: &QueryBidSchema,
        distinct: Option<&DistinctBidSchema>,
    ) -> anyhow::Result<i64>;

    async fn fetch_total_collection_offer(
        &self,
        collection_id: &str,
    ) -> anyhow::Result<Option<BigDecimal>>;
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

    async fn fetch_total_bids(
        &self,
        query: &QueryBidSchema,
        distinct: Option<&DistinctBidSchema>,
    ) -> anyhow::Result<i64> {
        let res = create_count_query_builder("bids", Schema::Bids, query, distinct)
            .build_query_scalar()
            .fetch_optional(&*self.pool)
            .await
            .context("Failed to fetch total bids")?;

        Ok(res.unwrap_or_default())
    }

    async fn fetch_total_collection_offer(
        &self,
        collection_id: &str,
    ) -> anyhow::Result<Option<BigDecimal>> {
        let collection_id = Uuid::from_str(collection_id).ok();
        let res = sqlx::query_scalar!(
            r#"
            SELECT SUM(b.price)
            FROM bids b
            WHERE b.collection_id = $1
                AND b.status = 'active'
                AND b.expired_at > NOW()
            GROUP BY b.collection_id
            "#,
            collection_id,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count filtered collections")?;

        Ok(res)
    }
}
