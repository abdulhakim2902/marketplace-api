use std::sync::Arc;

use crate::database::Schema;
use crate::models::schema::listing::{OrderListingSchema, QueryListingSchema};
use crate::models::{db::listing::DbListing, schema::listing::ListingSchema};
use crate::utils::schema::{handle_join, handle_nested_order, handle_order, handle_query};
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
        let mut builder = QueryBuilder::<Postgres>::new("");

        let selection_builder = QueryBuilder::<Postgres>::new(" SELECT * FROM listings ");
        let mut join_builder = QueryBuilder::<Postgres>::new("");
        let mut query_builder = QueryBuilder::<Postgres>::new("");
        let mut order_by_builder = QueryBuilder::<Postgres>::new("");

        // Handle join
        let order_map = structs::to_map(&order).ok().flatten();
        if let Some(object) = order_map.as_ref() {
            builder.push(" WITH ");
            handle_nested_order(&mut builder, object);
            if builder.sql().trim().ends_with("WITH") {
                builder.reset();
            } else {
                handle_join(&mut join_builder, object);
            }
        }

        // Handle query
        if let Some(object) = structs::to_map(&query).ok().flatten() {
            query_builder.push(" WHERE ");
            handle_query(&mut query_builder, &object, "AND", Schema::Listings);
            if query_builder.sql().trim().ends_with("WHERE") {
                query_builder.reset();
            }
        }

        // Handle ordering
        if let Some(object) = order_map.as_ref() {
            order_by_builder.push(" ORDER BY ");
            handle_order(&mut order_by_builder, object);
            if order_by_builder.sql().trim().ends_with("ORDER BY") {
                order_by_builder.reset();
            }
        }

        let pagination = format!(" LIMIT {} OFFSET {}", limit, offset);

        builder.push(selection_builder.sql());
        builder.push(join_builder.sql());
        builder.push(query_builder.sql());
        builder.push(order_by_builder.sql());
        builder.push(pagination);

        let res = builder
            .build_query_as::<ListingSchema>()
            .fetch_all(&*self.pool)
            .await
            .context("Failed to fetch bids")?;

        Ok(res)
    }
}
