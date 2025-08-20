use std::collections::HashMap;
use std::{str::FromStr, sync::Arc};

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
        limit: i64,
        offset: i64,
        query: &QueryNftSchema,
        order: &OrderNftSchema,
    ) -> anyhow::Result<i64>;

    async fn fetch_nft_uri(&self, offset: i64, limit: i64) -> anyhow::Result<Vec<DbNftUri>>;

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
                description,
                burned,
                version,
                royalty,
                updated_at,
                uri,
                token_id
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.id.clone());
            b.push_bind(item.name.clone());
            b.push_bind(item.owner.clone());
            b.push_bind(item.collection_id.clone());
            b.push_bind(item.properties.clone());
            b.push_bind(item.description.clone());
            b.push_bind(item.burned);
            b.push_bind(item.version.clone());
            b.push_bind(item.royalty.clone());
            b.push_bind(Utc::now());
            b.push_bind(item.uri.clone());
            b.push_bind(item.token_id);
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
                owner = EXCLUDED.owner,
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
        distinct: &DistinctNftSchema,
        limit: i64,
        offset: i64,
        query: &QueryNftSchema,
        order: &OrderNftSchema,
    ) -> anyhow::Result<Vec<NftSchema>> {
        let mut builder = QueryBuilder::<Postgres>::new(
            r#"
            WITH
                nft_rarities AS (
                    SELECT
                        na.collection_id,
                        na.nft_id,
                        SUM(-LOG(2, na.rarity))             AS rarity
                    FROM attributes na
                    GROUP BY na.collection_id, na.nft_id
                ),
                nfts AS (
                    SELECT 
                        n.id,
                        COALESCE(n.name, nm.name)                   AS name,
                        owner,
                        n.collection_id,
                        burned,
                        n.properties,
                        n.token_id,
                        COALESCE(n.description, nm.description)     AS description,
                        COALESCE(nm.image, n.uri)                   AS image_url,
                        nm.animation_url,
                        nm.avatar_url,
                        nm.youtube_url,
                        nm.external_url,
                        nm.background_color,
                        royalty,
                        version,
                        nr.rarity,
                        CASE
                            WHEN nr.rarity IS NOT NULL
                            THEN RANK () OVER (
                                PARTITION BY n.collection_id
                                ORDER BY nr.rarity DESC
                            )
                            END                                     AS ranking
                    FROM nfts n
                        LEFT JOIN nft_metadata nm ON nm.uri = n.uri AND nm.collection_id = n.collection_id
                        LEFT JOIN nft_rarities nr ON nr.nft_id = n.id AND nr.collection_id = n.collection_id
                )
            "#,
        );

        let selection_builder = QueryBuilder::<Postgres>::new(format!(
            " SELECT DISTINCT ON ({}) * FROM nfts ",
            distinct.to_string()
        ));

        let mut join_builder = QueryBuilder::<Postgres>::new("");
        let mut query_builder = QueryBuilder::<Postgres>::new("");
        let mut order_by_builder = QueryBuilder::<Postgres>::new("");

        // Handle join
        let order_map = structs::to_map(order).ok().flatten();
        if let Some(object) = order_map.as_ref() {
            let mut nested_order_builder = QueryBuilder::<Postgres>::new(",");
            handle_nested_order(&mut nested_order_builder, object);
            if !nested_order_builder.sql().ends_with(",") {
                builder.push(nested_order_builder.sql());
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
            order_by_builder.push(" ORDER BY ");
            handle_order(&mut order_by_builder, object);
            if order_by_builder.sql().trim().ends_with("ORDER BY") {
                order_by_builder.reset();
            }
        }

        let pagination = format!(" LIMIT {} OFFSET {}", limit, offset);

        builder.push(selection_builder.sql());
        builder.push(join_builder.sql());
        builder.push(query_builder.sql());
        builder.push(order_by_builder.sql());
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
        limit: i64,
        offset: i64,
        query: &QueryNftSchema,
        order: &OrderNftSchema,
    ) -> anyhow::Result<i64> {
        let mut builder = QueryBuilder::<Postgres>::new(
            r#"
            WITH
                nft_rarities AS (
                    SELECT
                        na.collection_id,
                        na.nft_id,
                        SUM(-LOG(2, na.rarity))             AS rarity
                    FROM attributes na
                    GROUP BY na.collection_id, na.nft_id
                ),
                nfts AS (
                    SELECT 
                        n.id,
                        COALESCE(n.name, nm.name)                   AS name,
                        owner,
                        n.collection_id,
                        burned,
                        n.properties,
                        n.token_id,
                        COALESCE(n.description, nm.description)     AS description,
                        COALESCE(nm.image, n.uri)                   AS image_url,
                        nm.animation_url,
                        nm.avatar_url,
                        nm.youtube_url,
                        nm.external_url,
                        nm.background_color,
                        royalty,
                        version,
                        nr.rarity,
                        CASE
                            WHEN nr.rarity IS NOT NULL
                            THEN RANK () OVER (
                                PARTITION BY n.collection_id
                                ORDER BY nr.rarity DESC
                            )
                            END                                     AS ranking
                    FROM nfts n
                        LEFT JOIN nft_metadata nm ON nm.uri = n.uri AND nm.collection_id = n.collection_id
                        LEFT JOIN nft_rarities nr ON nr.nft_id = n.id AND nr.collection_id = n.collection_id
                )
            "#,
        );

        let selection_builder = QueryBuilder::<Postgres>::new(format!(
            " SELECT COUNT(DISTINCT {}) FROM nfts ",
            distinct.to_string()
        ));

        let mut join_builder = QueryBuilder::<Postgres>::new("");
        let mut query_builder = QueryBuilder::<Postgres>::new("");
        let mut order_by_builder = QueryBuilder::<Postgres>::new("");

        // Handle join
        let order_map = structs::to_map(order).ok().flatten();
        if let Some(object) = order_map.as_ref() {
            let mut nested_order_builder = QueryBuilder::<Postgres>::new(",");
            handle_nested_order(&mut nested_order_builder, object);
            if !nested_order_builder.sql().ends_with(",") {
                builder.push(nested_order_builder.sql());
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
            order_by_builder.push(" ORDER BY ");
            handle_order(&mut order_by_builder, object);
            if order_by_builder.sql().trim().ends_with("ORDER BY") {
                order_by_builder.reset();
            }
        }

        let pagination = format!(" LIMIT {} OFFSET {}", limit, offset);

        builder.push(selection_builder.sql());
        builder.push(join_builder.sql());
        builder.push(query_builder.sql());
        builder.push(order_by_builder.sql());
        builder.push(pagination);

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

    async fn fetch_total_nft(
        &self,
        wallet_address: &str,
        collection_id: &str,
    ) -> anyhow::Result<i64> {
        let collection_id = Uuid::from_str(collection_id).ok();
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

impl Loader<Uuid> for Nfts {
    type Value = NftSchema;
    type Error = FieldError;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let res = sqlx::query_as!(
            NftSchema,
            r#"
             WITH
                nft_rarities AS (
                    SELECT
                        na.collection_id,
                        na.nft_id,
                        SUM(-LOG(2, na.rarity))             AS rarity
                    FROM attributes na
                    GROUP BY na.collection_id, na.nft_id
                )
            SELECT 
                n.id,
                COALESCE(n.name, nm.name)                   AS name,
                owner,
                n.collection_id,
                burned,
                n.properties,
                n.token_id,
                COALESCE(n.description, nm.description)     AS description,
                COALESCE(nm.image, n.uri)                   AS image_url,
                nm.animation_url,
                nm.avatar_url,
                nm.youtube_url,
                nm.external_url,
                nm.background_color,
                royalty,
                version,
                nr.rarity,
                CASE
                    WHEN nr.rarity IS NOT NULL
                    THEN RANK () OVER (
                        PARTITION BY n.collection_id
                        ORDER BY nr.rarity DESC
                    )
                    END                                     AS ranking
            FROM nfts n
                LEFT JOIN nft_metadata nm ON nm.uri = n.uri AND nm.collection_id = n.collection_id
                LEFT JOIN nft_rarities nr ON nr.nft_id = n.id AND nr.collection_id = n.collection_id
            WHERE n.id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(res.into_iter().map(|c| (c.id, c)).collect())
    }
}
