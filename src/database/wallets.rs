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
                latest_prices AS (
                    SELECT DISTINCT ON (tp.token_address) tp.token_address, tp.price FROM token_prices tp
                    WHERE tp.token_address = '0x000000000000000000000000000000000000000000000000000000000000000a'
                    ORDER BY tp.token_address, tp.created_at DESC
                ),
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
                                COALESCE(EXTRACT(EPOCH FROM sa.block_time), EXTRACT(EPOCH FROM ra.block_time))
                                    - EXTRACT(EPOCH FROM ra.block_time)
                                
                            )                                 AS holding_periods,
                            SUM(sa.price - ra.price)          AS profit                                                         
                        FROM activities ra
                            LEFT JOIN activities sa ON ra.receiver = sa.sender AND ra.nft_id = sa.nft_id AND ra.collection_id = sa.collection_id
                        WHERE ra.receiver = $1 
                            AND ra.sender IS NOT NULL
                            AND ra.tx_type IN ('transfer', 'buy', 'mint')
                        GROUP BY ra.receiver
                ),
                sale_activities AS (
                        SELECT a.sender AS address, COUNT(*) FROM activities a
                        WHERE a.sender = $1 AND a.tx_type = 'buy'
                        GROUP BY a.sender
                ),
                mint_activities AS (
                        SELECT a.receiver AS address, COUNT(*) FROM activities a
                        WHERE a.receiver = $1 AND a.tx_type = 'mint'
                        GROUP BY a.receiver
                )
            SELECT
                wn.count                AS unique_nfts,
                ta.total_buys,
                ta.trade_volumes,
                ta.holding_periods,
                sa.count                AS total_sales,
                ma.count                AS total_mints,
                ta.profit               AS total_profits,
                ta.profit * lp.price    AS total_usd_profits
            FROM wallets w
                LEFT JOIN wallet_nfts wn ON wn.owner = w.address
                LEFT JOIN traded_activities ta ON ta.address = w.address
                LEFT JOIN sale_activities sa ON sa.address = w.address 
                LEFT JOIN mint_activities ma ON ma.address = w.address
                LEFT JOIN latest_prices lp ON TRUE
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
                COALESCE(EXTRACT(EPOCH FROM sa.block_time), EXTRACT(EPOCH FROM ra.block_time)) -
                    EXTRACT(EPOCH FROM ra.block_time) AS period 
            FROM activities ra
                LEFT JOIN activities sa ON ra.receiver = sa.sender AND ra.nft_id = sa.nft_id AND ra.collection_id = sa.collection_id
            WHERE ra.receiver IS NOT NULL AND ra.receiver = $1 AND ra.tx_type IN ('transfer', 'buy', 'mint')
            ORDER BY period DESC
            LIMIT 10
            "#,
            address
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch nft holding periods")?;

        Ok(res)
    }
}
