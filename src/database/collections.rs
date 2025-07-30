use std::sync::Arc;

use anyhow::Context;
use sqlx::{
    PgPool, Postgres, QueryBuilder, Transaction,
    postgres::{PgQueryResult, types::PgInterval},
};

use crate::models::{
    api::responses::{
        collection::Collection, collection_nft_change::CollectionNftChange,
        collection_offer::CollectionOffer,
        collection_profit_leaderboard::CollectionProfitLeaderboard,
        collection_trending::CollectionTrending,
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

    async fn fetch_collections(
        &self,
        id: Option<String>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<Collection>>;

    async fn fetch_collection_trending(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionTrending>>;

    async fn fetch_collection_offers(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionOffer>>;

    async fn count_collection_offers(&self, id: &str) -> anyhow::Result<i64>;

    async fn fetch_collection_profit_leaderboard(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionProfitLeaderboard>>;

    async fn count_collection_profit_leaderboard(&self, id: &str) -> anyhow::Result<i64>;

    async fn fetch_collection_nft_change(
        &self,
        id: &str,
        interval: Option<PgInterval>,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionNftChange>>;

    async fn count_collection_nft_change(
        &self,
        id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<i64>;
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
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<Collection>> {
        let res = sqlx::query_as!(
            Collection,
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
            WHERE $1::TEXT IS NULL OR $1::TEXT = '' OR c.id = $1::TEXT 
            ORDER BY s.volume DESC
            LIMIT $2 OFFSET $3
            "#,
            id,
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
    ) -> anyhow::Result<Vec<CollectionTrending>> {
        let res = sqlx::query_as!(
            CollectionTrending,
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

    async fn fetch_collection_offers(
        &self,
        _id: &str,
        _page: i64,
        _size: i64,
    ) -> anyhow::Result<Vec<CollectionOffer>> {
        Ok(Vec::new())
    }

    async fn count_collection_offers(&self, _id: &str) -> anyhow::Result<i64> {
        Ok(10)
    }

    async fn fetch_collection_profit_leaderboard(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionProfitLeaderboard>> {
        let res = sqlx::query_as!(
            CollectionProfitLeaderboard,
            r#"
            WITH
                bought_activities AS (
                    SELECT a.collection_id, a.receiver AS address, COUNT(*) AS bought, SUM(price) AS price FROM activities a
                    WHERE a.tx_type = 'buy' AND a.collection_id = $1
                    GROUP BY a.collection_id, a.receiver 
                ),
                sold_activities AS (
                    SELECT a.collection_id, a.sender AS address, COUNT(*) AS sold, SUM(price) AS price FROM activities a
                    WHERE a.tx_type = 'buy' AND a.collection_id = $1
                    GROUP BY a.collection_id, a.sender
                ),
                unique_addresses AS (
                    SELECT ba.address FROM bought_activities ba
                    UNION
                    SELECT sa.address FROM sold_activities sa
                )
            SELECT
                ua.address,
                ba.bought, 
                sa.sold, 
                ba.price                                                                AS spent,
                (COALESCE(sa.price, 0) - COALESCE(ba.price, 0)) 	                    AS total_profit,
                (COALESCE(sa.price, 0) - COALESCE(ba.price, 0)) / NULLIF (ba.price, 0) 	AS profit_percentage
            FROM unique_addresses ua
                LEFT JOIN bought_activities ba ON ba.address = ua.address
                LEFT JOIN sold_activities sa ON sa.address = ua.address
            WHERE ua.address IS NOT NULL
            ORDER BY total_profit DESC
            LIMIT $2 OFFSET $3
            "#,
            id,
            size,
            size * (page - 1),
        ).fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection profit leaders")?;

        Ok(res)
    }

    async fn count_collection_profit_leaderboard(&self, id: &str) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            WITH addresses AS (
                SELECT a.receiver AS address FROM activities a
                WHERE a.tx_type = 'buy' AND a.collection_id = $1
                GROUP BY a.collection_id, a.receiver 
                UNION
                SELECT a.sender AS address FROM activities a
                WHERE a.tx_type = 'buy' AND a.collection_id = $1
                GROUP BY a.collection_id, a.sender
            )
            SELECT COUNT(*) FROM addresses
            WHERE addresses.address IS NOT NULL
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count collection profit leaderboard")?;

        Ok(res.unwrap_or_default())
    }

    async fn fetch_collection_nft_change(
        &self,
        id: &str,
        interval: Option<PgInterval>,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<CollectionNftChange>> {
        let res = sqlx::query_as!(
            CollectionNftChange,
            r#"
            WITH 
                current_nft_owners AS (
                    SELECT n.owner, COUNT(*) FROM nfts n
                    WHERE n.burned IS NULL OR NOT n.burned AND n.collection_id = $1
                    GROUP BY n.collection_id, n.owner
                ),
                transfer_in AS (
                    SELECT a.collection_id, a.receiver AS address, COUNT(*) FROM activities a
                    WHERE a.block_time >= NOW() - $2::INTERVAL 
                        AND a.tx_type = 'transfer'
                        AND a.collection_id = $1
                    GROUP BY a.collection_id, a.receiver
                ),
                transfer_out AS (
                    SELECT a.collection_id, a.sender AS address, COUNT(*) FROM activities a
                    WHERE a.block_time >= NOW() - $2::INTERVAL 
                        AND a.tx_type = 'transfer'
                        AND a.collection_id = $1
                    GROUP BY a.collection_id, a.sender
                ),
                unique_addresses AS (
                    SELECT tin.address FROM transfer_in tin
                    UNION
                    SELECT tout.address FROM transfer_out tout
                )
            SELECT 
                ua.address, 
                (COALESCE(tout.count, 0) - COALESCE(tin.count, 0)) 	AS change,
                COALESCE(co.count, 0) 								AS quantity	
            FROM unique_addresses ua
                LEFT JOIN transfer_in tin ON tin.address = ua.address
                LEFT JOIN transfer_out tout ON tout.address = ua.address
                LEFT JOIN current_nft_owners co ON co.owner = ua.address
            WHERE ua.address IS NOT NULL
            ORDER BY change DESC
            LIMIT $3 OFFSET $4
            "#,
            id,
            interval,
            size,
            size * (page - 1),
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection profit leaders")?;

        Ok(res)
    }

    async fn count_collection_nft_change(
        &self,
        id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            WITH addresses AS (
                SELECT a.receiver AS address FROM activities a
                WHERE a.block_time >= NOW() - $2::INTERVAL 
                    AND a.tx_type = 'transfer'
                    AND a.collection_id = $1
                GROUP BY a.collection_id, a.receiver
                UNION
                SELECT a.sender AS address FROM activities a
                WHERE a.block_time >= NOW() - $2::INTERVAL 
                    AND a.tx_type = 'transfer'
                    AND a.collection_id = $1
                GROUP BY a.collection_id, a.sender
            )
            SELECT COUNT(*) FROM addresses
            WHERE addresses.address IS NOT NULL
            "#,
            id,
            interval,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count collection profit leaderboard")?;

        Ok(res.unwrap_or_default())
    }
}
