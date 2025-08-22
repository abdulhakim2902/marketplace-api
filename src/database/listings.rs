use std::sync::Arc;

use crate::database::Schema;
use crate::models::schema::listing::{
    DistinctListingSchema, OrderListingSchema, QueryListingSchema,
};
use crate::models::{db::listing::DbListing, schema::listing::ListingSchema};
use crate::utils::schema::{create_count_query_builder, create_query_builder};
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
        query: &QueryListingSchema,
        order: &OrderListingSchema,
        distinct: Option<&DistinctListingSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ListingSchema>>;

    async fn fetch_total_listings(
        &self,
        query: &QueryListingSchema,
        distinct: Option<&DistinctListingSchema>,
    ) -> anyhow::Result<i64>;
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
        query: &QueryListingSchema,
        order: &OrderListingSchema,
        distinct: Option<&DistinctListingSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ListingSchema>> {
        create_query_builder(
            "listings",
            Schema::Listings,
            query,
            order,
            distinct,
            limit,
            offset,
        )
        .build_query_as::<ListingSchema>()
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch listings")
    }

    async fn fetch_total_listings(
        &self,
        query: &QueryListingSchema,
        distinct: Option<&DistinctListingSchema>,
    ) -> anyhow::Result<i64> {
        let res = create_count_query_builder("listings", Schema::Listings, query, distinct)
            .build_query_scalar()
            .fetch_optional(&*self.pool)
            .await
            .context("Failed to total listings")?;

        Ok(res.unwrap_or_default())
    }
}
