use std::{str::FromStr, sync::Arc};

use crate::models::{
    db::bid::DbBid,
    schema::bid::{BidSchema, OrderBidSchema, QueryBidSchema},
};
use crate::utils::schema::{handle_order, handle_query};
use crate::utils::structs;
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
        limit: i64,
        offset: i64,
        query: QueryBidSchema,
        order: OrderBidSchema,
    ) -> anyhow::Result<Vec<BidSchema>>;

    async fn fetch_collection_top_offer(
        &self,
        collection_id: &str,
    ) -> anyhow::Result<Option<BigDecimal>>;

    async fn fetch_nft_top_offer(&self, nft_id: &str) -> anyhow::Result<Option<BigDecimal>>;

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
        limit: i64,
        offset: i64,
        query: QueryBidSchema,
        order: OrderBidSchema,
    ) -> anyhow::Result<Vec<BidSchema>> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT * FROM bids
            WHERE
            "#,
        );

        if let Some(object) = structs::to_map(&query).ok().flatten() {
            handle_query(&mut query_builder, &object, "AND");
        }

        if query_builder.sql().trim().ends_with("WHERE") {
            query_builder.push(" ");
            query_builder.push_bind(true);
        }

        query_builder.push(" ORDER BY ");

        if let Some(object) = structs::to_map(&order).ok().flatten() {
            handle_order(&mut query_builder, &object);
        }

        if query_builder.sql().trim().ends_with("ORDER BY") {
            query_builder.push("updated_at");
        }

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let res = query_builder
            .build_query_as::<BidSchema>()
            .fetch_all(&*self.pool)
            .await
            .context("Failed to fetch activities")?;

        Ok(res)
    }

    async fn fetch_collection_top_offer(
        &self,
        collection_id: &str,
    ) -> anyhow::Result<Option<BigDecimal>> {
        let collection_id = Uuid::from_str(collection_id).ok();
        let res = sqlx::query_scalar!(
            r#"
            SELECT MAX(b.price)
            FROM bids b
            WHERE b.collection_id = $1
                AND b.status = 'active'
                AND b.bid_type = 'solo'
                AND b.expired_at > NOW()
            GROUP BY b.collection_id
            "#,
            collection_id,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count filtered collections")?;

        Ok(res.map(|e| BigDecimal::from(e)))
    }

    async fn fetch_nft_top_offer(&self, nft_id: &str) -> anyhow::Result<Option<BigDecimal>> {
        let nft_id = Uuid::from_str(nft_id).ok();
        let res = sqlx::query_scalar!(
            r#"
            SELECT MAX(b.price)
            FROM bids b
            WHERE b.nft_id = $1
                AND b.status = 'active'
                AND b.bid_type = 'solo'
                AND b.expired_at > NOW()
            GROUP BY b.nft_id
            "#,
            nft_id,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count filtered collections")?;

        Ok(res.map(|e| BigDecimal::from(e)))
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
