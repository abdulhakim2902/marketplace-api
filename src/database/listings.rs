use std::{str::FromStr, sync::Arc};

use crate::models::schema::listing::FilterListingSchema;
use crate::models::{db::listing::DbListing, schema::listing::ListingSchema};
use anyhow::Context;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait IListings: Send + Sync {
    async fn tx_insert_listings(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        listings: Vec<DbListing>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_listings(
        &self,
        filter: Option<FilterListingSchema>,
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
                id,
                block_height,
                block_time,
                market_contract_id, 
                collection_id, 
                nft_id, 
                listed,
                market_name, 
                nonce,
                price,
                seller, 
                tx_index
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.id);
            b.push_bind(item.block_height);
            b.push_bind(item.block_time);
            b.push_bind(item.market_contract_id.clone());
            b.push_bind(item.collection_id.clone());
            b.push_bind(item.nft_id.clone());
            b.push_bind(item.listed);
            b.push_bind(item.market_name.clone());
            b.push_bind(item.nonce);
            b.push_bind(item.price);
            b.push_bind(item.seller.clone());
            b.push_bind(item.tx_index);
        })
        .push(
            r#"
            ON CONFLICT (id) DO UPDATE SET 
                block_height = EXCLUDED.block_height,
                block_time = EXCLUDED.block_time,
                price = EXCLUDED.price,
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
        filter: Option<FilterListingSchema>,
    ) -> anyhow::Result<Vec<ListingSchema>> {
        let filter = filter.unwrap_or_default();

        let query = filter.where_.unwrap_or_default();
        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        let listing_id = query.id.map(|e| Uuid::from_str(&e).ok()).flatten();
        let nft_id = query.nft_id.map(|e| Uuid::from_str(&e).ok()).flatten();

        let res = sqlx::query_as!(
            ListingSchema,
            r#"
            SELECT * FROM listings l
            WHERE ($1::UUID IS NULL OR l.id = $1)
                AND ($2::UUID IS NULL OR l.nft_id = $2) 
                AND $3::BOOL IS NULL OR l.listed = $3
            LIMIT $4 OFFSET $5
            "#,
            listing_id,
            nft_id,
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
