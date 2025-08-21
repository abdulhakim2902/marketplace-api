use std::{str::FromStr, sync::Arc};

use crate::database::Schema;
use crate::models::db::attribute::DbAttribute;
use crate::models::schema::attribute::{
    AttributeSchema, DistinctAttributeSchema, OrderAttributeSchema, QueryAttributeSchema,
};
use crate::utils::schema::{handle_join, handle_nested_order, handle_order, handle_query};
use crate::utils::structs;
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
        distinct: &DistinctAttributeSchema,
        limit: i64,
        offset: i64,
        query: &QueryAttributeSchema,
        order: &OrderAttributeSchema,
    ) -> anyhow::Result<Vec<AttributeSchema>>;

    async fn fetch_total_attributes(
        &self,
        distinct: &DistinctAttributeSchema,
        query: &QueryAttributeSchema,
    ) -> anyhow::Result<i64>;

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
                id,
                collection_id,
                nft_id,
                attr_type,
                value
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.id.clone());
            b.push_bind(item.collection_id.clone());
            b.push_bind(item.nft_id.clone());
            b.push_bind(item.attr_type.clone());
            b.push_bind(item.value.clone());
        })
        .push(
            r#"
            ON CONFLICT (id) DO NOTHING
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
        distinct: &DistinctAttributeSchema,
        limit: i64,
        offset: i64,
        query: &QueryAttributeSchema,
        order: &OrderAttributeSchema,
    ) -> anyhow::Result<Vec<AttributeSchema>> {
        let mut builder = QueryBuilder::<Postgres>::new("");

        let mut selection_builder = QueryBuilder::<Postgres>::new("");
        let mut join_builder = QueryBuilder::<Postgres>::new("");
        let mut query_builder = QueryBuilder::<Postgres>::new("");
        let mut order_by_builder = QueryBuilder::<Postgres>::new("");

        // Handle selection
        if let DistinctAttributeSchema::None = distinct {
            selection_builder.push(" SELECT * FROM attributes ");
        } else {
            selection_builder.push(format!(
                " SELECT DISTINCT ON ({}) * FROM attributes ",
                distinct.to_string()
            ));
        }

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
            handle_query(&mut query_builder, &object, "AND", Schema::Attributes);
            if query_builder.sql().trim().ends_with("WHERE") {
                query_builder.reset();
            }
        }

        // Handle ordering
        if let Some(object) = order_map.as_ref() {
            if let DistinctAttributeSchema::None = distinct {
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
            .build_query_as::<AttributeSchema>()
            .fetch_all(&*self.pool)
            .await
            .context("Failed to fetch attributes")?;

        Ok(res)
    }

    async fn fetch_total_attributes(
        &self,
        distinct: &DistinctAttributeSchema,
        query: &QueryAttributeSchema,
    ) -> anyhow::Result<i64> {
        let mut builder = QueryBuilder::<Postgres>::new("");

        let mut selection_builder = QueryBuilder::<Postgres>::new("");
        let mut query_builder = QueryBuilder::<Postgres>::new("");

        // Handle selection
        if let DistinctAttributeSchema::None = distinct {
            selection_builder.push(" SELECT * FROM attributes ");
        } else {
            selection_builder.push(format!(
                " SELECT DISTINCT ON ({}) * FROM attributes ",
                distinct.to_string()
            ));
        }

        // Handle query
        if let Some(object) = structs::to_map(query).ok().flatten() {
            query_builder.push(" WHERE ");
            handle_query(&mut query_builder, &object, "AND", Schema::Attributes);
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
            .context("Failed to fetch total attributes")?;

        Ok(res.unwrap_or_default())
    }

    async fn collection_score(&self, collection_id: &str) -> anyhow::Result<Option<BigDecimal>> {
        let collection_id = Uuid::from_str(collection_id).ok();
        let res = sqlx::query_scalar!(
            r#"
            WITH collection_score AS (
                SELECT DISTINCT ON (collection_id, attr_type, value)
                    collection_id,
                    attr_type,
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
        let collection_id = Uuid::from_str(collection_id).ok();
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM (
                SELECT collection_id, attr_type FROM attributes
                WHERE collection_id = $1
                GROUP BY collection_id, attr_type
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
