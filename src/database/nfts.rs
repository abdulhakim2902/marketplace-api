use std::collections::HashMap;
use std::sync::Arc;

use crate::database::Schema;
use crate::models::schema::nft::{DistinctNftSchema, OrderNftSchema, QueryNftSchema};
use crate::models::{
    db::nft::{DbNft, DbNftUri},
    schema::nft::NftSchema,
};
use crate::utils::schema::{handle_join, handle_nested_order, handle_order, handle_query};
use crate::utils::structs;
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
        distinct: &DistinctNftSchema,
        limit: i64,
        offset: i64,
        query: &QueryNftSchema,
        order: &OrderNftSchema,
    ) -> anyhow::Result<Vec<NftSchema>>;

    async fn fetch_total_nfts(
        &self,
        distinct: &DistinctNftSchema,
        query: &QueryNftSchema,
    ) -> anyhow::Result<i64>;

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
                burned = EXCLUDED.burned,
                updated_at = EXCLUDED.updated_at,
                media_url = EXCLUDED.media_url,
                avatar_url = EXCLUDED.avatar_url,
                youtube_url = EXCLUDED.youtube_url,
                external_url = EXCLUDED.external_url,
                background_color = EXCLUDED.background_color
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
        distinct: &DistinctNftSchema,
        limit: i64,
        offset: i64,
        query: &QueryNftSchema,
        order: &OrderNftSchema,
    ) -> anyhow::Result<Vec<NftSchema>> {
        let mut builder = QueryBuilder::<Postgres>::new("");

        let mut selection_builder = QueryBuilder::<Postgres>::new("");
        let mut join_builder = QueryBuilder::<Postgres>::new("");
        let mut query_builder = QueryBuilder::<Postgres>::new("");
        let mut order_by_builder = QueryBuilder::<Postgres>::new("");

        // Handle selection
        if let DistinctNftSchema::None = distinct {
            selection_builder.push(
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
                "#,
            );
        } else {
            selection_builder.push(format!(
                r#"
                SELECT DISTINCT ON ({}) 
                    nfts.*,
                    CASE
                        WHEN rarity IS NOT NULL
                        THEN RANK () OVER (
                            PARTITION BY collection_id
                            ORDER BY rarity DESC
                        )
                    END                                 AS ranking
                FROM nfts
                "#,
                distinct.to_string(),
            ));
        };

        // Handle join
        let order_map = structs::to_map(order).ok().flatten();
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
        if let Some(object) = structs::to_map(query).ok().flatten() {
            query_builder.push(" WHERE ");
            handle_query(&mut query_builder, &object, "AND", Schema::Nfts);
            if query_builder.sql().trim().ends_with("WHERE") {
                query_builder.reset();
            }
        }

        // Handle ordering
        if let Some(object) = order_map.as_ref() {
            if let DistinctNftSchema::None = distinct {
                order_by_builder.push(" ORDER BY ");
            } else {
                order_by_builder.push(format!(" ORDER BY {}, ", distinct.to_string()));
            }

            handle_order(&mut order_by_builder, object);
            if order_by_builder.sql().trim().ends_with("ORDER BY") {
                order_by_builder.reset();
            }
        }

        let pagination = format!(" LIMIT {} OFFSET {}", limit, offset);

        builder.push(selection_builder.sql());
        builder.push(join_builder.sql());
        builder.push(query_builder.sql());
        builder.push(order_by_builder.sql().trim().trim_end_matches(","));
        builder.push(pagination);

        let res = builder
            .build_query_as::<NftSchema>()
            .fetch_all(&*self.pool)
            .await
            .context("Failed to fetch nfts")?;

        Ok(res)
    }

    async fn fetch_total_nfts(
        &self,
        distinct: &DistinctNftSchema,
        query: &QueryNftSchema,
    ) -> anyhow::Result<i64> {
        let mut builder = QueryBuilder::<Postgres>::new("");

        let mut selection_builder = QueryBuilder::<Postgres>::new("");
        let mut query_builder = QueryBuilder::<Postgres>::new("");

        // Handle selection
        if let DistinctNftSchema::None = distinct {
            selection_builder.push(
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
                "#,
            );
        } else {
            selection_builder.push(format!(
                r#"
                SELECT DISTINCT ON ({}) 
                    nfts.*,
                    CASE
                        WHEN rarity IS NOT NULL
                        THEN RANK () OVER (
                            PARTITION BY collection_id
                            ORDER BY rarity DESC
                        )
                    END                                 AS ranking
                FROM nfts
                "#,
                distinct.to_string(),
            ));
        };

        // Handle query
        if let Some(object) = structs::to_map(query).ok().flatten() {
            query_builder.push(" WHERE ");
            handle_query(&mut query_builder, &object, "AND", Schema::Nfts);
            if query_builder.sql().trim().ends_with("WHERE") {
                query_builder.reset();
            }
        }

        builder.push(selection_builder.sql());
        builder.push(query_builder.sql());

        let res = builder
            .build_query_scalar()
            .fetch_optional(&*self.pool)
            .await
            .context("Failed to fetch total nfts")?;

        Ok(res.unwrap_or_default())
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
