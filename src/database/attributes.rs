use std::{str::FromStr, sync::Arc};

use crate::models::db::attribute::DbAttribute;
use crate::models::schema::attribute::{AttributeSchema, FilterAttributeSchema};
use anyhow::Context;
use bigdecimal::BigDecimal;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait IAttributes: Send + Sync {
    async fn tx_insert_attributes(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbAttribute>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_attributes(
        &self,
        filter: FilterAttributeSchema,
    ) -> anyhow::Result<Vec<AttributeSchema>>;

    async fn collection_score(&self, collection_id: &str) -> anyhow::Result<Option<BigDecimal>>;

    async fn total_collection_trait(&self, collection_id: &str) -> anyhow::Result<i64>;

    async fn total_nft_trait(&self, collection_id: &str) -> anyhow::Result<i64>;
}

pub struct Attributes {
    pool: Arc<PgPool>,
}

impl Attributes {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl IAttributes for Attributes {
    async fn tx_insert_attributes(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbAttribute>,
    ) -> anyhow::Result<PgQueryResult> {
        if items.is_empty() {
            return Ok(PgQueryResult::default());
        }

        let res = QueryBuilder::<Postgres>::new(
            r#"
            INSERT INTO attributes (
                collection_id,
                nft_id,
                type,
                value
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.collection_id.clone());
            b.push_bind(item.nft_id.clone());
            b.push_bind(item.attr_type.clone());
            b.push_bind(item.value.clone());
        })
        .push(
            r#"
            ON CONFLICT (collection_id, nft_id, type, value) DO NOTHING
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert attributes")?;

        Ok(res)
    }

    async fn fetch_attributes(
        &self,
        filter: FilterAttributeSchema,
    ) -> anyhow::Result<Vec<AttributeSchema>> {
        let query = filter.where_.unwrap_or_default();
        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        let nft_id = query.nft_id.map(|e| Uuid::from_str(&e).ok()).flatten();

        let res = sqlx::query_as!(
            AttributeSchema,
            r#"
            SELECT
                na.collection_id,
                na.nft_id,
                na.type                 AS attr_type,
                na.value,
                na.rarity,
                na.score
            FROM attributes na
            WHERE ($1::TEXT IS NULL OR $1::TEXT = '' OR na.collection_id = $1)
                AND ($2::UUID IS NULL OR na.nft_id = $2)
            LIMIT $3 OFFSET $4
            "#,
            query.collection_id,
            nft_id,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch attributes")?;

        Ok(res)
    }

    async fn collection_score(&self, collection_id: &str) -> anyhow::Result<Option<BigDecimal>> {
        let res = sqlx::query_scalar!(
            r#"
            WITH collection_score AS (
                SELECT DISTINCT ON (collection_id, type, value)
                    collection_id,
                    type,
                    value,
                    rarity,
                    score
                FROM attributes
                WHERE collection_id = $1
            )
            SELECT SUM(ca.score)
            FROM attributes ca
            GROUP BY ca.collection_id
            "#,
            collection_id,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to calculate collection rarity")?;

        Ok(res)
    }

    async fn total_collection_trait(&self, collection_id: &str) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM (
                SELECT collection_id, type FROM attributes
                WHERE collection_id = $1
                GROUP BY collection_id, type
            ) AS collection_attributes
            "#,
            collection_id
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch collection trait")?;

        Ok(res.unwrap_or_default())
    }

    async fn total_nft_trait(&self, nft_id: &str) -> anyhow::Result<i64> {
        let nft_id = Uuid::from_str(nft_id).ok();
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM attributes
            WHERE nft_id = $1
            "#,
            nft_id,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch nft trait")?;

        Ok(res.unwrap_or_default())
    }
}
