use std::collections::HashMap;
use std::sync::Arc;

use crate::database::Schema;
use crate::models::schema::AggregateFieldsSchema;
use crate::models::schema::nft::{
    AggregateNftFieldsSchema, DistinctNftSchema, OrderNftSchema, QueryNftSchema,
};
use crate::models::{
    db::nft::{DbNft, DbNftUri},
    schema::nft::NftSchema,
};
use crate::utils::schema::{create_aggregate_query_builder, create_query_builder};
use anyhow::Context;
use async_graphql::FieldError;
use async_graphql::dataloader::Loader;
use chrono::Utc;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait INfts: Send + Sync {
    async fn tx_insert_nfts(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbNft>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_nfts(
        &self,
        query: &QueryNftSchema,
        order: &OrderNftSchema,
        distinct: Option<&DistinctNftSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<NftSchema>>;

    async fn fetch_aggregate_nfts(
        &self,
        selection: &HashMap<String, Vec<String>>,
        query: &QueryNftSchema,
        distinct: Option<&DistinctNftSchema>,
    ) -> anyhow::Result<AggregateFieldsSchema<AggregateNftFieldsSchema>>;

    async fn fetch_nft_uri(&self, offset: i64, limit: i64) -> anyhow::Result<Vec<DbNftUri>>;
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
                description,
                burned,
                version,
                royalty,
                updated_at,
                uri,
                token_id,
                media_url,
                animation_url,
                avatar_url,
                youtube_url,
                external_url,
                background_color
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.id);
            b.push_bind(item.name);
            b.push_bind(item.owner);
            b.push_bind(item.collection_id);
            b.push_bind(item.properties);
            b.push_bind(item.description);
            b.push_bind(item.burned);
            b.push_bind(item.version);
            b.push_bind(item.royalty);
            b.push_bind(Utc::now());
            b.push_bind(item.uri);
            b.push_bind(item.token_id);
            b.push_bind(item.media_url);
            b.push_bind(item.animation_url);
            b.push_bind(item.avatar_url);
            b.push_bind(item.youtube_url);
            b.push_bind(item.external_url);
            b.push_bind(item.background_color);
        })
        .push(
            r#"
            ON CONFLICT (id) DO UPDATE SET
                name = COALESCE(EXCLUDED.name, nfts.name),
                uri = COALESCE(EXCLUDED.uri, nfts.uri),
                description = COALESCE(EXCLUDED.description, nfts.description),
                properties = COALESCE(EXCLUDED.properties, nfts.properties),
                royalty = COALESCE(EXCLUDED.royalty, nfts.royalty),
                token_id = COALESCE(EXCLUDED.token_id, nfts.token_id),
                owner = CASE 
                    WHEN EXCLUDED.burned THEN NULL
                    ELSE COALESCE(EXCLUDED.owner, nfts.owner)
                END,
                burned = COALESCE(EXCLUDED.burned, nfts.burned),
                updated_at = EXCLUDED.updated_at,
                media_url = COALESCE(EXCLUDED.media_url, nfts.media_url),
                avatar_url = COALESCE(EXCLUDED.avatar_url, nfts.avatar_url),
                youtube_url = COALESCE(EXCLUDED.youtube_url, nfts.youtube_url),
                external_url = COALESCE(EXCLUDED.external_url, nfts.external_url),
                background_color = COALESCE(EXCLUDED.background_color, nfts.background_color)
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
        query: &QueryNftSchema,
        order: &OrderNftSchema,
        distinct: Option<&DistinctNftSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<NftSchema>> {
        create_query_builder(
            r#"
            (
                SELECT
                    nfts.*,
                    CASE
                        WHEN rarity IS NOT NULL
                        THEN RANK () OVER (
                            PARTITION BY collection_id
                            ORDER BY rarity DESC
                        )
                    END                                 AS ranking
                FROM nfts
            )
            "#,
            Schema::Nfts,
            query,
            order,
            distinct,
            limit,
            offset,
        )
        .build_query_as::<NftSchema>()
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch nfts")
    }

    async fn fetch_aggregate_nfts(
        &self,
        selection: &HashMap<String, Vec<String>>,
        query: &QueryNftSchema,
        distinct: Option<&DistinctNftSchema>,
    ) -> anyhow::Result<AggregateFieldsSchema<AggregateNftFieldsSchema>> {
        if selection.is_empty() {
            return Ok(AggregateFieldsSchema::default());
        }

        let table = if let Some(distinct) = distinct {
            format!(
                r#"
                (
                    SELECT DISTINCT ON ({}) * FROM (
                        SELECT
                            nfts.*,
                            CASE
                                WHEN rarity IS NOT NULL
                                THEN RANK () OVER (
                                    PARTITION BY collection_id
                                    ORDER BY rarity DESC
                                )
                            END                                 AS ranking
                        FROM nfts
                    )
                )"#,
                distinct
            )
        } else {
            format!(
                r#"
                (
                    SELECT
                        nfts.*,
                        CASE
                            WHEN rarity IS NOT NULL
                            THEN RANK () OVER (
                                PARTITION BY collection_id
                                ORDER BY rarity DESC
                            )
                        END                                 AS ranking
                    FROM nfts
                )
                "#
            )
        };

        let value = create_aggregate_query_builder(table.as_str(), selection, Schema::Nfts, query)
            .build_query_scalar::<serde_json::Value>()
            .fetch_one(&*self.pool)
            .await
            .context("Failed to fetch aggregate nfts")?;

        let result =
            serde_json::from_value::<AggregateFieldsSchema<AggregateNftFieldsSchema>>(value)
                .context("Failed to parse aggregate result")?;

        Ok(result)
    }

    async fn fetch_nft_uri(&self, offset: i64, limit: i64) -> anyhow::Result<Vec<DbNftUri>> {
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
}

impl Loader<Uuid> for Nfts {
    type Value = NftSchema;
    type Error = FieldError;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let res = sqlx::query_as!(
            NftSchema,
            r#"
            SELECT 
                nfts.*,
                CASE
                    WHEN rarity IS NOT NULL
                    THEN RANK () OVER (
                        PARTITION BY collection_id
                        ORDER BY rarity DESC
                    )
                END                                 AS ranking
            FROM nfts
            WHERE id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(res.into_iter().map(|c| (c.id, c)).collect())
    }
}
