use std::sync::Arc;

use crate::models::{
    api::responses::attribute::Attribute, db::attribute::Attribute as DbAttribute,
};
use anyhow::Context;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};

#[async_trait::async_trait]
pub trait IAttributes: Send + Sync {
    async fn tx_insert_attributes(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbAttribute>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_attributes(&self, page: i64, size: i64) -> anyhow::Result<Vec<Attribute>>;
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
                attr_type,
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
            ON CONFLICT (collection_id, nft_id, attr_type, value) DO NOTHING
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert attributes")?;

        Ok(res)
    }

    async fn fetch_attributes(&self, page: i64, size: i64) -> anyhow::Result<Vec<Attribute>> {
        let res = sqlx::query_as!(
            Attribute,
            r#"
            WITH
                collection_nfts AS (
                    SELECT nfts.collection_id, COUNT(*) FROM nfts
                    GROUP BY nfts.collection_id
                ),
                collection_attributes AS (
                    SELECT atr.collection_id, atr.attr_type, atr.value, COUNT(*) FROM attributes atr
                        JOIN collection_nfts cn ON cn.collection_id = atr.collection_id
                    GROUP by atr.collection_id, atr.attr_type, atr.value
                ),
                collection_rarities AS (
                    SELECT
                        ca.collection_id,
                        ca.attr_type, 
                        ca.value, 
                        (ca.count / cn.count) AS rarity,
                        -log(2, ca.count / cn.count) AS score
                    FROM collection_attributes ca
                        JOIN collection_nfts cn ON ca.collection_id = cn.collection_id
                )
            SELECT
                attr.collection_id,
                attr.nft_id,
                attr.attr_type,
                attr.value,
                cr.rarity::NUMERIC,
                cr.score::NUMERIC
            FROM attributes attr
                JOIN collection_rarities cr ON cr.collection_id = attr.collection_id 
                                                    AND cr.attr_type = attr.attr_type
                                                    AND cr.value = attr.value
            LIMIT $1 OFFSET $2
            "#,
            size,
            size * (page - 1),
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch attributes")?;

        Ok(res)
    }
}
