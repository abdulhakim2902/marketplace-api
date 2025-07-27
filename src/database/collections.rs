use std::sync::Arc;

use anyhow::Context;
use sqlx::{
    PgPool, Postgres, QueryBuilder, Transaction,
    postgres::{PgQueryResult, types::PgInterval},
};

use crate::models::{
    api::responses::{
        collection::Collection, collection_activity::CollectionActivity,
        collection_info::CollectionInfo, collection_nft::CollectionNft,
    },
    db::collection::Collection as DbCollection,
};

#[async_trait::async_trait]
pub trait ICollections: Send + Sync {
    async fn tx_insert_collections(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbCollection>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn filter(
        &self,
        interval: Option<PgInterval>,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<Collection>>;

    async fn count(&self) -> anyhow::Result<i64>;

    async fn fetch_collection_info(&self, id: &str) -> anyhow::Result<CollectionInfo>;

    async fn fetch_collection_nfts(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionNft>>;

    async fn count_collection_nfts(&self, id: &str) -> anyhow::Result<i64>;

    async fn fetch_collection_activities(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionActivity>>;

    async fn count_collection_activities(&self, id: &str) -> anyhow::Result<i64>;
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
                cover_url
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
                cover_url = EXCLUDED.cover_url
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert collections")?;

        Ok(res)
    }

    async fn filter(
        &self,
        interval: Option<PgInterval>,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<Collection>> {
        let res = sqlx::query_as!(
            Collection,
            r#"
            WITH 
                owners AS (
                    SELECT n.collection_id, COUNT(DISTINCT n.owner) AS owners
                    FROM nfts n
                    GROUP BY n.collection_id
                ),
                prev_floors AS (
                    SELECT DISTINCT ON (a.collection_id) a.collection_id, a.price FROM activities a
                    WHERE a.tx_type = 'list'
                        AND ($1::INTERVAL IS NULL OR a.block_time >= NOW() - $1::INTERVAL)
                    ORDER BY a.collection_id, a.price ASC
                ),
                listings AS (
                    SELECT l.collection_id, MIN(l.price) AS floor, COUNT(*) AS count
                    FROM listings l
                    WHERE l.listed
                    GROUP BY l.collection_id
                ),
                sales AS (
                    SELECT 
                        a.collection_id, 
                        COUNT(*)            AS count, 
                        SUM(a.price)        AS volume,
                        SUM(a.usd_price)    AS volume_usd
                    FROM activities a
                    WHERE a.tx_type = 'buy'
                        AND ($1::INTERVAL IS NULL OR a.block_time >= NOW() - $1::INTERVAL)
                    GROUP BY a.collection_id
                ),
                top_bids AS (
                    SELECT b.collection_id, MAX(b.price) AS price
                    FROM bids b
                    WHERE b.status = 'active'
                        AND b.bid_type = 'solo'
                        AND b.expires_at > NOW()
                    GROUP BY b.collection_id
                )
            SELECT
                c.id,
                c.slug, 
                c.supply, 
                c.title, 
                c.description, 
                c.cover_url, 
                l.floor,
                pf.price                                    AS prev_floor,
                l.count                                     AS listed,
                o.owners,
                s.count                                     AS sales,
                s.volume,
                s.volume_usd,
                tb.price                                    AS top_offer,
                (s.volume / NULLIF(s.count, 0))::NUMERIC    AS average 
            FROM collections c
                LEFT JOIN listings l ON c.id = l.collection_id
                LEFT JOIN owners o ON c.id = o.collection_id
                LEFT JOIN sales s ON c.id = s.collection_id
                LEFT JOIN top_bids tb ON c.id = tb.collection_id
                LEFT JOIN prev_floors pf ON c.id = pf.collection_id
            ORDER BY s.volume DESC
            LIMIT $2 OFFSET $3
            "#,
            interval,
            size,
            size * (page - 1),
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collections")?;

        Ok(res)
    }

    async fn count(&self) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM collections
            "#,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count filtered collections")?;

        Ok(res.unwrap_or_default())
    }

    async fn fetch_collection_info(&self, id: &str) -> anyhow::Result<CollectionInfo> {
        let res = sqlx::query_as!(
            CollectionInfo,
            r#"
            WITH 
                owners AS (
                    SELECT n.collection_id, COUNT(DISTINCT n.owner) AS owners
                    FROM nfts n
                    WHERE n.collection_id = $1
                    GROUP BY n.collection_id
                ),
                listings AS (
                    SELECT l.collection_id, MIN(l.price) AS floor, COUNT(*) AS count
                    FROM listings l
                    WHERE l.listed AND l.collection_id = $1
                    GROUP BY l.collection_id
                ),
                sales AS (
                    SELECT 
                        a.collection_id, 
                        COUNT(*)            AS count, 
                        SUM(a.price)        AS volume,
                        SUM(a.usd_price)    AS volume_usd
                    FROM activities a
                    WHERE a.tx_type = 'buy' AND a.collection_id = $1
                    GROUP BY a.collection_id
                ),
                sales_24h AS (
                    SELECT 
                        a.collection_id, 
                        COUNT(*)            AS count, 
                        SUM(a.price)        AS volume,
                        SUM(a.usd_price)    AS volume_usd
                    FROM activities a
                    WHERE a.tx_type = 'buy'
                        AND a.block_time >= NOW() - '1d'::INTERVAL
                    GROUP BY a.collection_id
                ),
                top_bids AS (
                    SELECT b.collection_id, MAX(b.price) AS price, SUM(b.price) AS total_offer
                    FROM bids b
                    WHERE b.status = 'active'
                        AND b.bid_type = 'solo'
                        AND b.expires_at > NOW()
                        AND b.collection_id = $1
                    GROUP BY b.collection_id
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
                l.floor,
                l.floor AS prev_floor,
                l.count                                     AS listed,
                o.owners,
                s.count                                     AS sales,
                s.volume                                    AS all_volume,
                s.volume_usd                                AS all_volume_usd,
                s24.count                                   AS sales_24h,
                s24.volume                                  AS volume_24h,
                s24.volume_usd                              AS volume_24h_usd,
                tb.price                                    AS top_offer,
                tb.total_offer                              AS total_offer,
                (s.volume / NULLIF(s.count, 0))::NUMERIC    AS average 
            FROM collections c
                LEFT JOIN listings l ON c.id = l.collection_id
                LEFT JOIN owners o ON c.id = o.collection_id
                LEFT JOIN sales s ON c.id = s.collection_id
                LEFT JOIN top_bids tb ON c.id = tb.collection_id
                LEFT JOIN sales_24h s24 ON c.id = s24.collection_id
            WHERE c.id = $1
            "#,
            id,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch collections")?;

        Ok(res)
    }

    async fn fetch_collection_nfts(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionNft>> {
        let res = sqlx::query_as!(
            CollectionNft,
            r#"
            WITH sales AS (
                SELECT DISTINCT ON (a.nft_id) 
                    a.nft_id, 
                    a.block_time, 
                    a.price 
                FROM activities a
                WHERE a.tx_type = 'buy'
                ORDER BY a.nft_id, a.block_time DESC
            )
            SELECT 
                n.id, 
                n.name, 
                n.image_url, 
                l.price AS listing_price, 
                s.price AS last_sale, 
                n.owner, 
                l.block_time AS listed_at 
            FROM nfts n
	            LEFT JOIN listings l ON l.nft_id = n.id AND l.listed
	            LEFT JOIN sales s ON s.nft_id = n.id
            WHERE n.collection_id = $1 AND n.burned IS NOT NULL AND NOT n.burned
            ORDER BY l.price
            LIMIT $2 OFFSET $3
            "#,
            id,
            size,
            size * (page - 1),
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch nfts")?;

        Ok(res)
    }

    async fn count_collection_nfts(&self, id: &str) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM nfts n
            WHERE n.collection_id = $1 AND n.burned IS NOT NULL AND NOT n.burned
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count filtered collection nfts")?;

        Ok(res.unwrap_or_default())
    }

    async fn fetch_collection_activities(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionActivity>> {
        let res = sqlx::query_as!(
            CollectionActivity,
            r#"
            SELECT 
                a.tx_type,
                a.tx_index,
                a.tx_id,
                a.sender                AS from,
                a.receiver              AS to,
                a.price,
                a.usd_price,
                a.market_name,
                a.market_contract_id,
                a.block_time            AS time,
                a.nft_id,
                n.name                  AS nft_name,
                n.description           AS nft_description,
                n.image_url             AS nft_image_url
            FROM activities a
	            LEFT JOIN nfts n ON n.id = a.nft_id
            WHERE a.collection_id = $1
            ORDER BY a.block_time
            LIMIT $2 OFFSET $3
            "#,
            id,
            size,
            size * (page - 1),
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch nfts")?;

        Ok(res)
    }

    async fn count_collection_activities(&self, id: &str) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM activities a
            WHERE a.collection_id = $1
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count filtered collection activities")?;

        Ok(res.unwrap_or_default())
    }
}
