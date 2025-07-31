use std::sync::Arc;

use anyhow::Context;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};

use crate::models::{
    db::collection::DbCollection,
    schema::{collection::CollectionSchema, collection_trending::CollectionTrendingSchema},
};

#[async_trait::async_trait]
pub trait ICollections: Send + Sync {
    async fn tx_insert_collections(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbCollection>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_collections(
        &self,
        id: Option<String>,
        wallet_address: Option<String>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<CollectionSchema>>;

    async fn fetch_collection_trending(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionTrendingSchema>>;
}

pub struct Collections {
    pool: Arc<PgPool>,
}

impl Collections {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ICollections for Collections {
    async fn tx_insert_collections(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbCollection>,
    ) -> anyhow::Result<PgQueryResult> {
        if items.is_empty() {
            return Ok(PgQueryResult::default());
        }

        let res = QueryBuilder::<Postgres>::new(
            r#"
            INSERT INTO collections (
                id,
                slug,
                title,
                supply,
                twitter,
                discord,
                website,
                verified,
                description,
                cover_url,
                royalty
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.id.clone());
            b.push_bind(item.slug.clone());
            b.push_bind(item.title.clone());
            b.push_bind(item.supply);
            b.push_bind(item.twitter.clone());
            b.push_bind(item.discord.clone());
            b.push_bind(item.website.clone());
            b.push_bind(item.verified);
            b.push_bind(item.description.clone());
            b.push_bind(item.cover_url.clone());
            b.push_bind(item.royalty.clone());
        })
        .push(
            r#"
            ON CONFLICT (id) DO UPDATE SET
                slug = EXCLUDED.slug,
                title = EXCLUDED.title,
                supply = EXCLUDED.supply,
                twitter = EXCLUDED.twitter,
                discord = EXCLUDED.discord,
                website = EXCLUDED.website,
                verified = EXCLUDED.verified,
                description = EXCLUDED.description,
                cover_url = EXCLUDED.cover_url,
                royalty = EXCLUDED.royalty
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert collections")?;

        Ok(res)
    }

    async fn fetch_collections(
        &self,
        id: Option<String>,
        wallet_address: Option<String>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<CollectionSchema>> {
        let res = sqlx::query_as!(
            CollectionSchema,
            r#"
            WITH 
                sales AS (
                    SELECT
                        a.collection_id, 
                        SUM(a.price)        AS volume,
                        COUNT(*)            AS total
                    FROM activities a
                    WHERE a.tx_type = 'buy'
                    GROUP BY a.collection_id
                ),
                listings AS (
                    SELECT 
                        l.collection_id,
                        MIN(l.price)        AS floor,
                        COUNT(*)            AS total
                    FROM listings l
                    WHERE l.listed
                    GROUP BY l.collection_id
                ),
                nft_owners AS (
                    SELECT n.collection_id, COUNT(DISTINCT n.owner) AS total
                    FROM nfts n
                    WHERE n.burned IS NULL OR NOT n.burned
                    GROUP BY n.collection_id
                ) 
            SELECT
                c.id,
                c.slug, 
                c.supply, 
                c.title, 
                c.description, 
                c.cover_url, 
                c.verified,
                c.website,
                c.discord,
                c.twitter,
                c.royalty,
                s.volume            AS total_volume,
                s.total             AS total_sale,
                no.total            AS total_owner,
                l.floor,
                l.total             AS listed
            FROM collections c
                LEFT JOIN sales s ON s.collection_id = c.id
                LEFT JOIN listings l ON l.collection_id = c.id
                LEFT JOIN nft_owners no ON no.collection_id = c.id
            WHERE ($1::TEXT IS NULL OR $1::TEXT = '' OR c.id = $1::TEXT)
                AND ($2::TEXT IS NULL OR $2::TEXT = '' OR c.id IN (
                    SELECT DISTINCT n.collection_id FROM nfts n
                    WHERE n.owner = $2
                ))
            ORDER BY s.volume DESC
            LIMIT $3 OFFSET $4
            "#,
            id,
            wallet_address,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collections")?;

        Ok(res)
    }

    async fn fetch_collection_trending(
        &self,
        id: &str,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<CollectionTrendingSchema>> {
        let res = sqlx::query_as!(
            CollectionTrendingSchema,
            r#"
            WITH 
                nft_activities AS (
                    SELECT a.nft_id, COUNT(*) FROM activities a
                    WHERE a.tx_type IN ('mint', 'buy', 'transfer') AND a.collection_id = $1
                    GROUP BY a.nft_id
                ),
                price_activities AS (
                    SELECT DISTINCT ON (a.nft_id) a.nft_id, a,price FROM activities a
                    WHERE a.tx_type IN ('mint', 'buy', 'transfer') 
                        AND a.collection_id = $1
                        AND a.price > 0
                    ORDER BY a.nft_id, a.block_time DESC
                )
            SELECT 
                n.id                AS nft_id,
                n.collection_id     AS collection_id,
                na.count            AS tx_frequency,
                pa.price            AS last_price
            FROM nfts n
                LEFT JOIN nft_activities na ON na.nft_id = n.id
                LEFT JOIN price_activities pa ON na.nft_id = n.id
            ORDER BY na.count DESC
            LIMIT $2 OFFSET $3
            "#,
            id,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection nft trendings")?;

        Ok(res)
    }
}
