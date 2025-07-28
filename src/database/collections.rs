use std::sync::Arc;

use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{
    PgPool, Postgres, QueryBuilder, Transaction,
    postgres::{PgQueryResult, types::PgInterval},
};

use crate::models::{
    api::responses::{
        collection::Collection, collection_activity::CollectionActivity,
        collection_info::CollectionInfo, collection_nft::CollectionNft,
        collection_nft_holder::CollectionNftHolder, collection_nft_trending::CollectionNftTrending,
        collection_top_buyer::CollectionTopBuyer, collection_top_seller::CollectionTopSeller,
        data_point::DataPoint,
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
        account: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionNft>>;

    async fn count_collection_nfts(&self, id: &str, account: &str) -> anyhow::Result<i64>;

    async fn fetch_collection_activities(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionActivity>>;

    async fn count_collection_activities(&self, id: &str) -> anyhow::Result<i64>;

    async fn fetch_collection_floor_chart(
        &self,
        id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        interval: PgInterval,
    ) -> anyhow::Result<Vec<DataPoint>>;

    async fn fetch_collection_top_buyers(
        &self,
        id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<CollectionTopBuyer>>;

    async fn fetch_collection_top_sellers(
        &self,
        id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<CollectionTopSeller>>;

    async fn fetch_collection_nft_holders(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionNftHolder>>;

    async fn count_collection_nft_holders(&self, id: &str) -> anyhow::Result<i64>;

    async fn fetch_collection_trending_nfts(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionNftTrending>>;

    async fn count_collection_trending_nfts(&self, id: &str) -> anyhow::Result<i64>;
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
                        AND b.expired_at > NOW()
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
                        AND b.expired_at > NOW()
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
        account: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionNft>> {
        let res = sqlx::query_as!(
            CollectionNft,
            r#"
            WITH
                listing_prices AS (
                    SELECT DISTINCT ON (l.nft_id) l.nft_id, l.price, l.block_time
                    FROM listings l
                    WHERE l.listed
                    ORDER BY l.nft_id, l.price ASC
                ),
                top_bids AS (
                    SELECT b.nft_id, MAX(b.price) AS price
                    FROM bids b
                    WHERE b.status = 'active'
                        AND b.bid_type = 'solo'
                        AND b.expired_at > NOW()
                    GROUP BY b.nft_id
                ),
                sales AS (
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
                n.owner, 
                n.description,
                lp.price            AS listing_price,
                s.price             AS last_sale, 
                lp.block_time       AS listed_at,
                tb.price            AS top_offer
            FROM nfts n
	            LEFT JOIN listing_prices lp ON lp.nft_id = n.id
	            LEFT JOIN sales s ON s.nft_id = n.id
                LEFT JOIN top_bids tb ON tb.nft_id = n.id
            WHERE n.collection_id = $1 
                AND NOT n.burned
                AND ($2 = '' OR n.owner = $2)
            ORDER BY lp.price
            LIMIT $3 OFFSET $4
            "#,
            id,
            account,
            size,
            size * (page - 1),
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch nfts")?;

        Ok(res)
    }

    async fn count_collection_nfts(&self, id: &str, account: &str) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM nfts n
            WHERE n.collection_id = $1 
                AND NOT n.burned
                AND ($2 = '' OR n.owner = $2)
            "#,
            id,
            account,
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
        .context("Failed to fetch collection activities")?;

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

    async fn fetch_collection_floor_chart(
        &self,
        id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        interval: PgInterval,
    ) -> anyhow::Result<Vec<DataPoint>> {
        let res = sqlx::query_as!(
            DataPoint,
            r#"
            WITH 
                time_series AS (
                    SELECT GENERATE_SERIES($2::TIMESTAMPTZ, $3::TIMESTAMPTZ, $4::INTERVAL) AS time_bin
                ),
                floor_prices AS (
                    SELECT 
                        ts.time_bin AS time,
                        COALESCE(
                            (
                                SELECT a.price FROM activities a
                                WHERE a.tx_type = 'list'
                                    AND a.collection_id = $1
                                    AND a.block_time >= ts.time_bin AND a.block_time < ts.time_bin + $4::INTERVAL
                                ORDER BY a.price ASC
                                LIMIT 1
                            ),
                            0
                        ) AS floor
                    FROM time_series ts
                    ORDER BY ts.time_bin
                )
            SELECT 
                ts.time_bin AS x,
                COALESCE(
                    (
                        SELECT fp.floor FROM floor_prices fp
                        WHERE fp.time <= ts.time_bin
                        LIMIT 1
                    ),
                    (
                        SELECT a.price FROM activities a
                        WHERE a.tx_type = 'list'
                            AND a.collection_id = $1
                            AND a.block_time <= ts.time_bin
                        ORDER BY a.price ASC
                        LIMIT 1
                    ),
                    0
                ) AS y
            FROM time_series ts
            ORDER BY ts.time_bin
            "#,
            id,
            start_time,
            end_time,
            interval,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection floor chart")?;

        Ok(res)
    }

    async fn fetch_collection_top_buyers(
        &self,
        id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<CollectionTopBuyer>> {
        let res = sqlx::query_as!(
            CollectionTopBuyer,
            r#"
            SELECT 
                a.receiver      AS buyer, 
                COUNT(*)        AS bought, 
                SUM(a.price)    AS volume
            FROM activities a
            WHERE a.tx_type = 'buy'
                AND a.collection_id = $1
                AND ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL)
            GROUP BY a.receiver
            ORDER BY bought DESC, volume DESC
            LIMIT 10
            "#,
            id,
            interval,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection top buyers")?;

        Ok(res)
    }

    async fn fetch_collection_top_sellers(
        &self,
        id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<CollectionTopSeller>> {
        let res = sqlx::query_as!(
            CollectionTopSeller,
            r#"
            SELECT 
                a.sender        AS seller, 
                COUNT(*)        AS sold, 
                SUM(a.price)    AS volume
            FROM activities a
            WHERE a.tx_type = 'buy'
                AND a.collection_id = $1
                AND ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL)
            GROUP BY a.sender
            ORDER BY sold DESC, volume DESC
            LIMIT 10
            "#,
            id,
            interval,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection top sellers")?;

        Ok(res)
    }

    async fn fetch_collection_nft_holders(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionNftHolder>> {
        let res = sqlx::query_as!(
            CollectionNftHolder,
            r#"
            WITH 
                mint_activities AS (
                    SELECT
                        a.receiver  AS address, 
                        COUNT(*)    AS count
                    FROM activities a
                    WHERE a.tx_type = 'mint' AND a.collection_id = $1
                    GROUP BY a.receiver
                ),
                send_activities AS (
                    SELECT
                        a.sender    AS address, 
                        COUNT(*)    AS count
                    FROM activities a
                    WHERE a.tx_type = 'buy' AND a.collection_id = $1
                    GROUP BY a.sender
                ),
                receive_activities AS (
                    SELECT
                        a.receiver  AS address, 
                        COUNT(*)    AS count
                    FROM activities a
                    WHERE a.tx_type = 'buy' AND a.collection_id = $1
                    GROUP BY a.receiver
                ),
                nft_owners AS (
                    SELECT 
                        n.owner     AS address,
                        COUNT(*)    AS count
                    FROM nfts n
                    WHERE n.collection_id = $1 AND NOT n.burned
                    GROUP BY n.owner
                )
            SELECT 
                no.address, 
                no.count            AS quantity, 
                ma.count            AS mint,
                sa.count            AS send,
                ra.count            AS receive
            FROM nft_owners no
                LEFT JOIN mint_activities ma ON ma.address = no.address
                LEFT JOIN send_activities sa ON sa.address = no.address
                LEFT JOIN receive_activities ra ON ra.address = no.address
            ORDER BY no.count
            LIMIT $2 OFFSET $3
            "#,
            id,
            size,
            size * (page - 1),
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection nft holders")?;

        Ok(res)
    }

    async fn count_collection_nft_holders(&self, id: &str) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM nfts n
            WHERE n.collection_id = $1 AND NOT n.burned
            GROUP BY n.owner
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count filtered collections")?;

        Ok(res.unwrap_or_default())
    }

    async fn fetch_collection_trending_nfts(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionNftTrending>> {
        let res = sqlx::query_as!(
            CollectionNftTrending,
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
                n.name              AS nft_name,
                n.image_url         AS nft_image_url,
                na.count            AS tx_frequency,
                pa.price            AS last_price
            FROM nfts n
                LEFT JOIN nft_activities na ON na.nft_id = n.id
                LEFT JOIN price_activities pa ON na.nft_id = n.id
            ORDER BY na.count DESC
            LIMIT $2 OFFSET $3
            "#,
            id,
            size,
            size * (page - 1),
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection nft trendings")?;

        Ok(res)
    }

    async fn count_collection_trending_nfts(&self, id: &str) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM nfts n
            WHERE n.collection_id = $1 AND NOT n.burned
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count filtered collection nft trendings")?;

        Ok(res.unwrap_or_default())
    }
}
