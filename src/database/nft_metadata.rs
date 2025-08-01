use std::sync::Arc;

use anyhow::Context;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};

use crate::models::db::nft_metadata::DbNFTMetadata;

#[async_trait::async_trait]
pub trait INFTMetadata: Send + Sync {
    async fn tx_insert_nft_metadata(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbNFTMetadata>,
    ) -> anyhow::Result<PgQueryResult>;
}

pub struct NFTMetadata {
    _pool: Arc<PgPool>,
}

impl NFTMetadata {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait::async_trait]
impl INFTMetadata for NFTMetadata {
    async fn tx_insert_nft_metadata(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbNFTMetadata>,
    ) -> anyhow::Result<PgQueryResult> {
        if items.is_empty() {
            return Ok(PgQueryResult::default());
        }

        let res = QueryBuilder::<Postgres>::new(
            r#"
            INSERT INTO nft_metadata (
                uri,
                collection_id,
                name,
                description,
                image,
                animation_url,
                avatar_url,
                background_color,
                image_data,
                youtube_url,
                external_url,
                attributes,
                properties
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.uri.clone());
            b.push_bind(item.collection_id);
            b.push_bind(item.name.clone());
            b.push_bind(item.description.clone());
            b.push_bind(item.image.clone());
            b.push_bind(item.animation_url.clone());
            b.push_bind(item.avatar_url.clone());
            b.push_bind(item.background_color.clone());
            b.push_bind(item.image_data.clone());
            b.push_bind(item.youtube_url.clone());
            b.push_bind(item.external_url.clone());
            b.push_bind(item.attributes.clone());
            b.push_bind(item.properties.clone());
        })
        .push(
            r#"
            ON CONFLICT (uri, collection_id) DO NOTHING
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert collections")?;

        Ok(res)
    }
}
