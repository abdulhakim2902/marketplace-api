use std::{collections::HashMap, sync::Arc};

use crate::database::Schema;
use crate::models::db::attribute::DbAttribute;
use crate::models::schema::AggregateFieldsSchema;
use crate::models::schema::attribute::{
    AggregateAttributeFieldsSchema, AttributeSchema, DistinctAttributeSchema, OrderAttributeSchema,
    QueryAttributeSchema,
};
use crate::utils::schema::{create_aggregate_query_builder, create_query_builder};
use anyhow::Context;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};

#[async_trait::async_trait]
pub trait IAttributes: Send + Sync {
    async fn tx_insert_attributes(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbAttribute>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_attributes(
        &self,
        query: &QueryAttributeSchema,
        order: &OrderAttributeSchema,
        distinct: Option<&DistinctAttributeSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<AttributeSchema>>;

    async fn fetch_aggregate_attributes(
        &self,
        selection: &HashMap<String, Vec<String>>,
        query: &QueryAttributeSchema,
        distinct: Option<&DistinctAttributeSchema>,
    ) -> anyhow::Result<AggregateFieldsSchema<AggregateAttributeFieldsSchema>>;
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
        query: &QueryAttributeSchema,
        order: &OrderAttributeSchema,
        distinct: Option<&DistinctAttributeSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<AttributeSchema>> {
        create_query_builder(
            "attributes",
            Schema::Attributes,
            query,
            order,
            distinct,
            limit,
            offset,
        )
        .build_query_as::<AttributeSchema>()
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch attributes")
    }

    async fn fetch_aggregate_attributes(
        &self,
        selection: &HashMap<String, Vec<String>>,
        query: &QueryAttributeSchema,
        distinct: Option<&DistinctAttributeSchema>,
    ) -> anyhow::Result<AggregateFieldsSchema<AggregateAttributeFieldsSchema>> {
        if selection.is_empty() {
            return Ok(AggregateFieldsSchema::default());
        }

        let table = if let Some(distinct) = distinct {
            format!("(SELECT DISTINCT ON ({}) * FROM attributes)", distinct)
        } else {
            "(SELECT * FROM attributes)".to_string()
        };

        let value =
            create_aggregate_query_builder(table.as_str(), selection, Schema::Attributes, query)
                .build_query_scalar::<serde_json::Value>()
                .fetch_one(&*self.pool)
                .await
                .context("Failed to fetch aggregate attributes")?;

        let result =
            serde_json::from_value::<AggregateFieldsSchema<AggregateAttributeFieldsSchema>>(value)
                .context("Failed to parse aggregate result")?;

        Ok(result)
    }
}
