use std::sync::Arc;

use crate::models::db::activity::Activity;
use anyhow::Context;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};

#[async_trait::async_trait]
pub trait IActivities: Send + Sync {
    async fn tx_insert_activities(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<Activity>,
    ) -> anyhow::Result<PgQueryResult>;
}

pub struct Activities {
    _pool: Arc<PgPool>,
}

impl Activities {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait::async_trait]
impl IActivities for Activities {
    async fn tx_insert_activities(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<Activity>,
    ) -> anyhow::Result<PgQueryResult> {
        if items.is_empty() {
            return Ok(PgQueryResult::default());
        }

        let res = QueryBuilder::<Postgres>::new(
            r#"
            INSERT INTO activities (
                tx_type,
                tx_index,
                tx_id,
                sender,
                receiver,
                price,
                nft_id,
                collection_id,
                market_contract_id,
                market_name,
                usd_price,
                block_time,
                block_height,
                amount
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.tx_type);
            b.push_bind(item.tx_index);
            b.push_bind(item.tx_id.clone());
            b.push_bind(item.sender.clone());
            b.push_bind(item.receiver.clone());
            b.push_bind(item.price);
            b.push_bind(item.nft_id.clone());
            b.push_bind(item.collection_id.clone());
            b.push_bind(item.market_contract_id.clone());
            b.push_bind(item.market_name.clone());
            b.push_bind(item.usd_price);
            b.push_bind(item.block_time);
            b.push_bind(item.block_height);
            b.push_bind(item.amount);
        })
        .push(
            r#"
            ON CONFLICT (tx_index, tx_id) DO NOTHING
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert activities")?;

        Ok(res)
    }
}
