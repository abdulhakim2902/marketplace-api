use std::sync::Arc;

use crate::models::{
    db::listing::DbListing,
    schema::listing::{ListingSchema, WhereListingSchema},
};
use anyhow::Context;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};

#[async_trait::async_trait]
pub trait IListings: Send + Sync {
    async fn tx_insert_listings(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        listings: Vec<DbListing>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_listings(
        &self,
        query: &WhereListingSchema,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ListingSchema>>;
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

    async fn fetch_listings(
        &self,
        query: &WhereListingSchema,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ListingSchema>> {
        let res = sqlx::query_as!(
            ListingSchema,
            r#"
            WITH latest_prices AS (
                SELECT DISTINCT ON (tp.token_address) tp.token_address, tp.price FROM token_prices tp
                WHERE tp.token_address = '0x000000000000000000000000000000000000000000000000000000000000000a'
                ORDER BY tp.token_address, tp.created_at DESC
            )
            SELECT
                l.block_height,
                l.block_time,
                l.market_contract_id,
                l.listed,
                l.market_name,
                l.collection_id,
                l.nft_id,
                l.nonce,
                l.price,
                l.price * lp.price      AS usd_price,
                l.seller,
                l.tx_index
            FROM listings l
                LEFT JOIN latest_prices lp ON TRUE
            WHERE ($1::TEXT IS NULL OR $1 = '' OR l.nft_id = $1) 
                AND $2::BOOL IS NULL OR l.listed = $2
            LIMIT $3 OFFSET $4
            "#,
            query.nft_id,
            query.is_listed,
            limit,
           offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch nft listings")?;

        Ok(res)
    }
}
