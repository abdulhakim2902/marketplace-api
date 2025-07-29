use std::sync::Arc;

use crate::models::db::listing::Listing as DbListing;
use anyhow::Context;
use bigdecimal::BigDecimal;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};

#[async_trait::async_trait]
pub trait IListings: Send + Sync {
    async fn tx_insert_listings(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        listings: Vec<DbListing>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_collection_floor(
        &self,
        collection_id: &str,
    ) -> anyhow::Result<Option<BigDecimal>>;

    async fn fetch_total_listed(&self, collection_id: &str) -> anyhow::Result<i64>;
}

pub struct Listings {
    pool: Arc<PgPool>,
}

impl Listings {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl IListings for Listings {
    async fn tx_insert_listings(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbListing>,
    ) -> anyhow::Result<PgQueryResult> {
        if items.is_empty() {
            return Ok(PgQueryResult::default());
        }

        let res = QueryBuilder::<Postgres>::new(
            r#"
            INSERT INTO listings (
                block_height,
                block_time,
                market_contract_id, 
                collection_id, 
                nft_id, 
                listed,
                market_name, 
                nonce,
                price,
                price_str, 
                seller, 
                tx_index
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.block_height);
            b.push_bind(item.block_time);
            b.push_bind(item.market_contract_id.clone());
            b.push_bind(item.collection_id.clone());
            b.push_bind(item.nft_id.clone());
            b.push_bind(item.listed);
            b.push_bind(item.market_name.clone());
            b.push_bind(item.nonce);
            b.push_bind(item.price);
            b.push_bind(item.price_str.clone());
            b.push_bind(item.seller.clone());
            b.push_bind(item.tx_index);
        })
        .push(
            r#"
            ON CONFLICT (market_contract_id, nft_id) DO UPDATE SET 
                block_height = EXCLUDED.block_height,
                block_time = EXCLUDED.block_time,
                price = EXCLUDED.price,
                price_str = EXCLUDED.price_str, 
                listed = EXCLUDED.listed, 
                nonce = EXCLUDED.nonce,
                seller = EXCLUDED.seller,
                tx_index = EXCLUDED.tx_index
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert listings")?;

        Ok(res)
    }

    async fn fetch_collection_floor(
        &self,
        collection_id: &str,
    ) -> anyhow::Result<Option<BigDecimal>> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT MIN(l.price) AS floor
            FROM listings l
            WHERE l.listed AND l.collection_id = $1
            GROUP BY l.collection_id
            "#,
            collection_id
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch collection floor")?;

        Ok(res)
    }

    async fn fetch_total_listed(&self, collection_id: &str) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM listings l
            WHERE l.listed AND l.collection_id = $1
            GROUP BY l.collection_id
            "#,
            collection_id
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch total listed")?;

        Ok(res.unwrap_or_default())
    }
}
