use std::sync::Arc;

use crate::models::{db::bid::DbBid, schema::bid::BidSchema};
use anyhow::Context;
use bigdecimal::BigDecimal;
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
        collection_id: Option<String>,
        nft_id: Option<String>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<BidSchema>>;

    async fn fetch_collection_top_offer(
        &self,
        collection_id: &str,
    ) -> anyhow::Result<Option<BigDecimal>>;

    async fn fetch_nft_top_offer(&self, nft_id: &str) -> anyhow::Result<Option<BigDecimal>>;
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
                price_str, 
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
            b.push_bind(item.price_str.clone());
            b.push_bind(item.receiver.clone());
            b.push_bind(item.remaining_count);
            b.push_bind(item.status.clone());
            b.push_bind(item.bid_type.clone());
            b.push_bind(Utc::now());
        })
        .push(
            r#"
            ON CONFLICT (market_contract_id, id, bidder) DO UPDATE
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
        collection_id: Option<String>,
        nft_id: Option<String>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<BidSchema>> {
        let res = sqlx::query_as!(
            BidSchema,
            r#"
            WITH latest_prices AS (
                SELECT DISTINCT ON (tp.token_address) tp.token_address, tp.price FROM token_prices tp
                WHERE tp.token_address = '0x000000000000000000000000000000000000000000000000000000000000000a'
            )
            SELECT 
                b.*, 
                b.price * lp.price AS usd_price
            FROM bids b
                LEFT JOIN latest_prices lp ON TRUE
            WHERE ($1::TEXT IS NULL OR $1::TEXT = '' OR b.nft_id = $1)
                AND ($2::TEXT IS NULL OR $1::TEXT = '' OR b.collection_id = $2)
            LIMIT $3 OFFSET $4
            "#,
            nft_id,
            collection_id,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fecth bids")?;

        Ok(res)
    }

    async fn fetch_collection_top_offer(
        &self,
        collection_id: &str,
    ) -> anyhow::Result<Option<BigDecimal>> {
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

        Ok(res)
    }

    async fn fetch_nft_top_offer(&self, nft_id: &str) -> anyhow::Result<Option<BigDecimal>> {
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

        Ok(res)
    }
}
