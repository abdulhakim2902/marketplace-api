use std::sync::Arc;

use crate::models::db::token_price::DbTokenPrice;
use anyhow::Context;
use bigdecimal::BigDecimal;
use sqlx::{PgPool, postgres::PgQueryResult};

#[async_trait::async_trait]
pub trait ITokenPrices: Send + Sync {
    async fn insert_token_price(&self, token_price: &DbTokenPrice)
    -> anyhow::Result<PgQueryResult>;

    async fn get_token_price(&self, token_addr: &str) -> anyhow::Result<BigDecimal>;
}

pub struct TokenPrices {
    pool: Arc<PgPool>,
}

impl TokenPrices {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ITokenPrices for TokenPrices {
    async fn insert_token_price(
        &self,
        token_price: &DbTokenPrice,
    ) -> anyhow::Result<PgQueryResult> {
        let res = sqlx::query!(
            r#"
            INSERT INTO token_prices (token_address, price, created_at) 
            VALUES ($1, $2, $3)
            ON CONFLICT (token_address, created_at) DO NOTHING
            "#,
            token_price.token_address,
            token_price.price,
            token_price.created_at
        )
        .execute(&*self.pool)
        .await
        .context("Failed to insert token price")?;

        Ok(res)
    }

    async fn get_token_price(&self, token_addr: &str) -> anyhow::Result<BigDecimal> {
        let res = sqlx::query!(
            r#"
            SELECT tp.price FROM token_prices tp
            WHERE tp.token_address = $1
            ORDER BY tp.created_at DESC
            LIMIT 1
            "#,
            token_addr
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch token price")?;

        Ok(res.price)
    }
}
