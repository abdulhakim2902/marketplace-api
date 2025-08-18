use std::sync::Arc;

use crate::database::Schema;
use crate::models::schema::listing::{OrderListingSchema, QueryListingSchema};
use crate::models::{db::listing::DbListing, schema::listing::ListingSchema};
use crate::utils::schema::{handle_order, handle_query};
use crate::utils::structs;
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
        limit: i64,
        offset: i64,
        query: QueryListingSchema,
        order: OrderListingSchema,
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
        limit: i64,
        offset: i64,
        query: QueryListingSchema,
        order: OrderListingSchema,
    ) -> anyhow::Result<Vec<ListingSchema>> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT 
                id,
                block_height,
                block_time,
                market_contract_id,
                listed,
                market_name,
                nft_id,
                nonce,
                price,
                seller,
                tx_index
            FROM listings
            WHERE
            "#,
        );

        if let Some(object) = structs::to_map(&query).ok().flatten() {
            handle_query(&mut query_builder, &object, "AND", Schema::Listings);
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
            query_builder.push("block_time");
        }

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let res = query_builder
            .build_query_as::<ListingSchema>()
            .fetch_all(&*self.pool)
            .await
            .context("Failed to fetch bids")?;

        Ok(res)
    }
}
