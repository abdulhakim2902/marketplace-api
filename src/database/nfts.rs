use std::sync::Arc;

use crate::models::{
    db::nft::{DbNft, DbNftUri},
    schema::nft::{NftSchema, WhereNftSchema},
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

    async fn fetch_nfts(
        &self,
        query: &WhereNftSchema,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<NftSchema>>;

    async fn fetch_nft_uri(
        &self,
        offset: i64,
        limit: i64,
    ) -> anyhow::Result<Vec<DbNftUri>>;

    async fn fetch_total_nft(
        &self,
        wallet_address: &str,
        collection_id: &str,
    ) -> anyhow::Result<i64>;
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

    async fn fetch_nfts(
        &self,
        query: &WhereNftSchema,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<NftSchema>> {
        let res = sqlx::query_as!(
            NftSchema,
            r#"
            WITH
                latest_prices AS (
                    SELECT DISTINCT ON (tp.token_address) tp.token_address, tp.price FROM token_prices tp
                    WHERE tp.token_address = '0x000000000000000000000000000000000000000000000000000000000000000a'
                    ORDER BY tp.token_address, tp.created_at DESC
                ),
                listing_prices AS (
                    SELECT DISTINCT ON (l.nft_id) l.nft_id, l.price, l.block_time
                    FROM listings l
                    WHERE l.listed
                    ORDER BY l.nft_id, l.price ASC
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
                updated_at,
                lp.price                AS list_price,
                lp.price * ltp.price    AS list_usd_price,
                CASE
                WHEN lp.block_time IS NOT NULL
                    THEN lp.block_time
                    ELSE NULL
                END                     AS listed_at,
                s.price                 AS last_sale
            FROM nfts n
	            LEFT JOIN listing_prices lp ON lp.nft_id = n.id
	            LEFT JOIN sales s ON s.nft_id = n.id
                LEFT JOIN latest_prices ltp ON TRUE
            WHERE ($1::TEXT IS NULL OR $1::TEXT = '' OR n.id = $1) 
                AND ($2::TEXT IS NULL OR $2::TEXT = '' OR n.collection_id = $2) 
                AND ($3::TEXT IS NULL OR $3::TEXT = '' OR n.owner = $3) 
                AND (n.burned IS NULL OR NOT n.burned)
            ORDER BY lp.price
            LIMIT $4 OFFSET $5
            "#,
            query.nft_id,
            query.collection_id,
            query.wallet_address,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch nfts")?;

        Ok(res)
    }

    async fn fetch_nft_uri(
        &self,
        offset: i64,
        limit: i64,
    ) -> anyhow::Result<Vec<DbNftUri>> {
        let res = sqlx::query_as!(
            DbNftUri,
            r#"
            SELECT 
                n.collection_id, 
                n.uri, 
                jsonb_agg(DISTINCT n.id)    AS nft_ids,
                MIN(n.updated_at)           AS updated_at              
            FROM nfts n
                LEFT JOIN nft_metadata nm ON nm.uri = n.uri 
            WHERE n.uri ILIKE '%.json' AND nm.uri IS NULL
            GROUP BY n.collection_id, n.uri
            ORDER BY updated_at ASC
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

    async fn fetch_total_nft(
        &self,
        wallet_address: &str,
        collection_id: &str,
    ) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM nfts n
            WHERE n.owner = $1 AND n.collection_id = $2
            "#,
            wallet_address,
            collection_id,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count total nft")?;

        Ok(res.unwrap_or_default())
    }
}
