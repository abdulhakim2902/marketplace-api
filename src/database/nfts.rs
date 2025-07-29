use std::sync::Arc;

use crate::models::{
    api::responses::{nft::Nft, nft_info::NftInfo, nft_listing::NftListing, nft_offer::NftOffer},
    db::nft::Nft as DbNft,
};
use anyhow::Context;
use chrono::Utc;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};

#[async_trait::async_trait]
pub trait INfts: Send + Sync {
    async fn tx_insert_nfts(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbNft>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_nft_by_id(&self, id: &str) -> anyhow::Result<Nft>;

    async fn fetch_nft_info(&self, id: &str) -> anyhow::Result<NftInfo>;

    async fn fetch_nft_metadata_urls(&self, offset: i64, limit: i64) -> anyhow::Result<Vec<DbNft>>;

    async fn count_nft_metadata_urls(&self) -> anyhow::Result<i64>;

    async fn fetch_nft_listings(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<NftListing>>;

    async fn count_nft_listings(&self, id: &str) -> anyhow::Result<i64>;

    async fn fetch_nft_offers(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<NftOffer>>;

    async fn count_nft_offers(&self, id: &str) -> anyhow::Result<i64>;
}

pub struct Nfts {
    pool: Arc<PgPool>,
}

impl Nfts {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl INfts for Nfts {
    async fn tx_insert_nfts(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbNft>,
    ) -> anyhow::Result<PgQueryResult> {
        if items.is_empty() {
            return Ok(PgQueryResult::default());
        }

        let res = QueryBuilder::<Postgres>::new(
            r#"
            INSERT INTO nfts (
                id,
                name,
                owner,
                collection_id,
                properties,
                image_data,
                avatar_url,
                image_url,
                external_url,
                description,
                background_color,
                animation_url,
                youtube_url,
                burned,
                version,
                royalty,
                updated_at,
                uri
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.id.clone());
            b.push_bind(item.name.clone());
            b.push_bind(item.owner.clone());
            b.push_bind(item.collection_id.clone());
            b.push_bind(item.properties.clone());
            b.push_bind(item.image_data.clone());
            b.push_bind(item.avatar_url.clone());
            b.push_bind(item.image_url.clone());
            b.push_bind(item.external_url.clone());
            b.push_bind(item.description.clone());
            b.push_bind(item.background_color.clone());
            b.push_bind(item.animation_url.clone());
            b.push_bind(item.youtube_url.clone());
            b.push_bind(item.burned);
            b.push_bind(item.version.clone());
            b.push_bind(item.royalty.clone());
            b.push_bind(Utc::now());
            b.push_bind(item.uri);
        })
        .push(
            r#"
            ON CONFLICT (id) DO UPDATE SET
                owner = EXCLUDED.owner,
                name = COALESCE(EXCLUDED.name, nfts.name),
                uri = EXCLUDED.uri,
                image_url = COALESCE(EXCLUDED.image_url, nfts.image_url),
                description = COALESCE(EXCLUDED.description, nfts.description),
                properties = COALESCE(EXCLUDED.properties, nfts.properties),
                background_color = COALESCE(EXCLUDED.background_color, nfts.background_color),
                image_data = COALESCE(EXCLUDED.image_data, nfts.image_data),
                animation_url = COALESCE(EXCLUDED.animation_url, nfts.animation_url),
                youtube_url = COALESCE(EXCLUDED.youtube_url, nfts.youtube_url),
                avatar_url = COALESCE(EXCLUDED.avatar_url, nfts.avatar_url),
                external_url = COALESCE(EXCLUDED.external_url, nfts.external_url),
                royalty = COALESCE(EXCLUDED.royalty, nfts.royalty),
                burned = EXCLUDED.burned,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert nfts")?;

        Ok(res)
    }

    async fn fetch_nft_by_id(&self, id: &str) -> anyhow::Result<Nft> {
        let res = sqlx::query_as!(
            Nft,
            r#"
            SELECT
                id,
                name,
                owner,
                collection_id,
                burned,
                properties,
                description,
                background_color,
                image_data,
                animation_url,
                youtube_url,
                avatar_url,
                external_url,
                uri,
                image_url,
                royalty,
                version,
                updated_at
            FROM nfts n
            WHERE n.id = $1
            "#,
            id,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch nft info")?;

        Ok(res)
    }

    async fn fetch_nft_info(&self, id: &str) -> anyhow::Result<NftInfo> {
        let res = sqlx::query_as!(
            NftInfo,
            r#"
            WITH 
                listing_prices AS (
                    SELECT l.nft_id, MIN(l.price) AS price
                    FROM listings l
                    WHERE l.nft_id = $1 AND l.listed
                    GROUP BY l.nft_id
                ),
                top_bids AS (
                    SELECT b.nft_id, MAX(b.price) AS price
                    FROM bids b
                    WHERE b.status = 'active'
                        AND b.bid_type = 'solo'
                        AND b.expired_at > NOW()
                        AND b.nft_id = $1
                    GROUP BY b.nft_id
                )
            SELECT
                id,
                name,
                description,
                image_url,
                owner,
                tb.price        AS top_offer,
                lp.price        AS list_price
            FROM nfts n
                LEFT JOIN top_bids tb ON tb.nft_id = n.id
                LEFT JOIN listing_prices lp ON lp.nft_id = n.id
            WHERE n.id = $1 AND (n.burned IS NULL OR NOT n.burned)
            "#,
            id,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch nft info")?;

        Ok(res)
    }

    async fn fetch_nft_metadata_urls(&self, offset: i64, limit: i64) -> anyhow::Result<Vec<DbNft>> {
        let res = sqlx::query_as!(
            DbNft,
            r#"
            SELECT * FROM nfts
            WHERE nfts.uri ILIKE '%.json'
            ORDER BY nfts.updated_at ASC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch nft metadata urls")?;

        Ok(res)
    }

    async fn count_nft_metadata_urls(&self) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM nfts
            WHERE nfts.uri ILIKE '%.json'
            "#,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count nft metadata urls")?;

        Ok(res.unwrap_or_default())
    }

    async fn fetch_nft_listings(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<NftListing>> {
        let res = sqlx::query_as!(
            NftListing,
            r#"
            WITH latest_prices AS (
                SELECT DISTINCT ON (tp.token_address) tp.token_address, tp.price FROM token_prices tp
                WHERE tp.token_address = '0x000000000000000000000000000000000000000000000000000000000000000a'
                ORDER BY tp.token_address, tp.created_at DESC
            )
            SELECT
                l.price,
                l.market_name,
                l.market_contract_id,
                l.seller                                     AS from,
                l.price * lp.price                           AS usd_price
            FROM listings l
                LEFT JOIN latest_prices lp ON TRUE
            WHERE l.nft_id = $1 AND l.listed
            LIMIT $2 OFFSET $3
            "#,
            id,
            size,
            size * (page - 1),
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch nft listings")?;

        Ok(res)
    }

    async fn count_nft_listings(&self, id: &str) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM listings l
            WHERE l.nft_id = $1 AND l.listed
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count filtered nft listings")?;

        Ok(res.unwrap_or_default())
    }

    async fn fetch_nft_offers(
        &self,
        id: &str,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<NftOffer>> {
        let res = sqlx::query_as!(
            NftOffer,
            r#"
            WITH latest_prices AS (
                SELECT DISTINCT ON (tp.token_address) tp.token_address, tp.price FROM token_prices tp
                WHERE tp.token_address = '0x000000000000000000000000000000000000000000000000000000000000000a'
                ORDER BY tp.token_address, tp.created_at DESC
            )
            SELECT
                b.price,
                b.price * lp.price                          AS usd_price,
                b.bidder                                    AS from,
                b.expired_at,
                (
                    SELECT a.block_time FROM activities a
                    WHERE a.nft_id = $1 AND a.tx_id = b.created_tx_id
                    LIMIT 1
                )                                           AS updated_at,
                b.status
            FROM bids b
                LEFT JOIN latest_prices lp ON TRUE
            WHERE b.nft_id = $1
            ORDER by updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
            id,
            size,
            size * (page - 1),
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch nft offers")?;

        Ok(res)
    }

    async fn count_nft_offers(&self, id: &str) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM bids b
            WHERE b.nft_id = $1
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count filtered nft offers")?;

        Ok(res.unwrap_or_default())
    }
}
