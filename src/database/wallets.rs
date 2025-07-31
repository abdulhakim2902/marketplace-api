use std::sync::Arc;

use anyhow::Context;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};

use crate::models::schema::wallet::{nft_holding_period::NftHoldingPeriod, stats::StatsSchema};

#[async_trait::async_trait]
pub trait IWallets: Send + Sync {
    async fn tx_insert_wallets(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        wallets: Vec<String>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_stats(&self, address: &str) -> anyhow::Result<StatsSchema>;

    async fn fetch_nft_holding_periods(
        &self,
        address: &str,
    ) -> anyhow::Result<Vec<NftHoldingPeriod>>;
}

pub struct Wallets {
    pool: Arc<PgPool>,
}

impl Wallets {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl IWallets for Wallets {
    async fn tx_insert_wallets(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<String>,
    ) -> anyhow::Result<PgQueryResult> {
        if items.is_empty() {
            return Ok(PgQueryResult::default());
        }

        let res = QueryBuilder::<Postgres>::new(
            r#"
            INSERT INTO wallets (address)
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item);
        })
        .push(
            r#"
            ON CONFLICT (address) DO NOTHING
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert wallets")?;

        Ok(res)
    }

    async fn fetch_stats(&self, address: &str) -> anyhow::Result<StatsSchema> {
        let res = sqlx::query_as!(
            StatsSchema,
            r#"
            WITH
              wallet_nfts AS (
                  SELECT n.owner, COUNT(*) FROM nfts n
                  WHERE n.owner = $1
                  GROUP BY n.owner
              ),
              traded_activities AS (
                  SELECT
                      ra.receiver                       AS address, 
                      SUM(COALESCE(ra.usd_price, 0))    AS trade_volumes, 
                      SUM(
                          CASE
                              WHEN ra.tx_type = 'buy' OR (ra.tx_type = 'mint' AND ra.price > 0) THEN 1
                              ELSE 0 
                          END
                      )                                 AS total_buys,
                      AVG(
                        EXTRACT(EPOCH FROM ra.block_time) - 
                            COALESCE(EXTRACT(EPOCH FROM sa.block_time), EXTRACT(EPOCH FROM ra.block_time))
                      )                                 AS holding_periods
                  FROM activities ra
                      LEFT JOIN activities sa ON ra.sender = sa.receiver AND ra.nft_id = sa.nft_id AND ra.collection_id = sa.collection_id
                  WHERE ra.receiver = $1 
                      AND ra.sender IS NOT NULL
                      AND ra.tx_type IN ('transfer', 'buy', 'mint')
                  GROUP BY ra.receiver
              )
            SELECT
                wn.count            AS unique_nfts,
                ta.total_buys,
                ta.trade_volumes,
                ta.holding_periods
            FROM wallets w
                JOIN wallet_nfts wn ON wn.owner = w.address
                JOIN traded_activities ta ON ta.address = w.address
            WHERE w.address = $1
            "#,
            address,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed wallet stat")?;

        Ok(res)
    }

    async fn fetch_nft_holding_periods(
        &self,
        address: &str,
    ) -> anyhow::Result<Vec<NftHoldingPeriod>> {
        let res = sqlx::query_as!(
            NftHoldingPeriod,
            r#"
            SELECT
                ra.collection_id,
                ra.nft_id,
                EXTRACT(EPOCH FROM ra.block_time) - 
                    COALESCE(EXTRACT(EPOCH FROM sa.block_time), EXTRACT(EPOCH FROM ra.block_time)) AS period 
            FROM activities ra
                LEFT JOIN activities sa ON ra.sender = sa.receiver AND ra.nft_id = sa.nft_id AND ra.collection_id = sa.collection_id
            WHERE ra.receiver IS NOT NULL AND ra.receiver = $1 AND ra.tx_type IN ('transfer', 'buy', 'mint')
            ORDER BY period DESC
            "#,
            address
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch nft holding periods")?;

        Ok(res)
    }
}
