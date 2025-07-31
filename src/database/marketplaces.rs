use std::sync::Arc;

use anyhow::Context;
use sqlx::{PgPool, Postgres, QueryBuilder, postgres::PgQueryResult};

use crate::{
    config::marketplace_config::NFTMarketplaceConfig,
    models::schema::marketplace::MarketplaceSchema,
};

#[async_trait::async_trait]
pub trait IMarketplaces: Send + Sync {
    async fn insert_market_places(
        &self,
        items: &[NFTMarketplaceConfig],
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_marketplaces(&self) -> anyhow::Result<Vec<MarketplaceSchema>>;
}

pub struct Marketplaces {
    pool: Arc<PgPool>,
}

impl Marketplaces {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl IMarketplaces for Marketplaces {
    async fn insert_market_places(
        &self,
        items: &[NFTMarketplaceConfig],
    ) -> anyhow::Result<PgQueryResult> {
        if items.is_empty() {
            return Ok(PgQueryResult::default());
        }

        let res = QueryBuilder::<Postgres>::new(
            r#"
            INSERT INTO marketplaces (name, contract_address)
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.name.clone());
            b.push_bind(item.contract_address.clone());
        })
        .push(
            r#"
            ON CONFLICT (name, contract_address) DO NOTHING
            "#,
        )
        .build()
        .execute(&*self.pool)
        .await
        .context("Failed to insert marketplaces")?;

        Ok(res)
    }

    async fn fetch_marketplaces(&self) -> anyhow::Result<Vec<MarketplaceSchema>> {
        let res = sqlx::query_as!(MarketplaceSchema, r#"SELECT * FROM marketplaces"#)
            .fetch_all(&*self.pool)
            .await
            .context("Failed to fetch marketplaces")?;

        Ok(res)
    }
}
