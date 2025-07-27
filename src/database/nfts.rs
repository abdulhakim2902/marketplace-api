use std::sync::Arc;

use crate::models::db::nft::Nft;
use anyhow::Context;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};

#[async_trait::async_trait]
pub trait INfts: Send + Sync {
    async fn tx_insert_nfts(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<Nft>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_nft_metadata_urls(&self, offset: i64, limit: i64) -> anyhow::Result<Vec<Nft>>;

    async fn count_nft_metadata_urls(&self) -> anyhow::Result<i64>;
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
        items: Vec<Nft>,
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
                version
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
        })
        .push(
            r#"
            ON CONFLICT (id) DO UPDATE SET
                owner = EXCLUDED.owner,
                name = COALESCE(EXCLUDED.name, nfts.name),
                image_url = COALESCE(EXCLUDED.image_url, nfts.image_url),
                description = COALESCE(EXCLUDED.description, nfts.description),
                properties = COALESCE(EXCLUDED.properties, nfts.properties),
                background_color = COALESCE(EXCLUDED.background_color, nfts.background_color),
                image_data = COALESCE(EXCLUDED.image_data, nfts.image_data),
                animation_url = COALESCE(EXCLUDED.animation_url, nfts.animation_url),
                youtube_url = COALESCE(EXCLUDED.youtube_url, nfts.youtube_url),
                avatar_url = COALESCE(EXCLUDED.avatar_url, nfts.avatar_url),
                external_url = COALESCE(EXCLUDED.external_url, nfts.external_url),
                burned = EXCLUDED.burned
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert nfts")?;

        Ok(res)
    }

    async fn fetch_nft_metadata_urls(&self, offset: i64, limit: i64) -> anyhow::Result<Vec<Nft>> {
        let res = sqlx::query_as!(
            Nft,
            r#"
            SELECT * FROM nfts
            WHERE nfts.image_url ILIKE '$.json'
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
            WHERE nfts.image_url ILIKE '$.json'
            "#,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count nft metadata urls")?;

        Ok(res.unwrap_or_default())
    }
}
