use std::sync::Arc;

use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{
    PgPool, Postgres, QueryBuilder, Transaction,
    postgres::{PgQueryResult, types::PgInterval},
};

use crate::{
    models::{
        db::collection::DbCollection,
        schema::{
            collection::{
                CollectionSchema, OrderCollectionSchema, WhereCollectionSchema,
                attribute::AttributeSchema,
                nft_change::{NftChangeSchema, WhereNftChangeSchema},
                nft_distribution::{NftAmountDistributionSchema, NftPeriodDistributionSchema},
                nft_holder::NftHolderSchema,
                profit_leaderboard::{ProfitLeaderboardSchema, WhereLeaderboardSchema},
                top_buyer::TopBuyerSchema,
                top_seller::TopSellerSchema,
                trending::{TrendingSchema, WhereTrendingSchema},
            },
            data_point::DataPointSchema,
        },
    },
    utils::string_utils,
};

#[async_trait::async_trait]
pub trait ICollections: Send + Sync {
    async fn tx_insert_collections(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbCollection>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_collections(
        &self,
        query: &WhereCollectionSchema,
        order_by: Option<OrderCollectionSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<CollectionSchema>>;

    async fn fetch_trending(
        &self,
        query: &WhereTrendingSchema,
        page: i64,
        size: i64,
    ) -> anyhow::Result<Vec<TrendingSchema>>;

    async fn fetch_nft_changes(
        &self,
        query: &WhereNftChangeSchema,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<NftChangeSchema>>;

    async fn fetch_profit_leaderboards(
        &self,
        query: &WhereLeaderboardSchema,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ProfitLeaderboardSchema>>;

    async fn fetch_attributes(&self, collection_id: &str) -> anyhow::Result<Vec<AttributeSchema>>;

    async fn fetch_top_buyers(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<TopBuyerSchema>>;

    async fn fetch_top_sellers(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<TopSellerSchema>>;

    async fn fetch_nft_holders(
        &self,
        collection_id: &str,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<NftHolderSchema>>;

    async fn fetch_nft_amount_distribution(
        &self,
        collection_id: &str,
    ) -> anyhow::Result<NftAmountDistributionSchema>;

    async fn fetch_nft_period_distribution(
        &self,
        id: &str,
    ) -> anyhow::Result<NftPeriodDistributionSchema>;

    async fn fetch_floor_charts(
        &self,
        collection_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        interval: PgInterval,
    ) -> anyhow::Result<Vec<DataPointSchema>>;
}

pub struct Collections {
    pool: Arc<PgPool>,
}

impl Collections {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ICollections for Collections {
    async fn tx_insert_collections(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbCollection>,
    ) -> anyhow::Result<PgQueryResult> {
        if items.is_empty() {
            return Ok(PgQueryResult::default());
        }

        let res = QueryBuilder::<Postgres>::new(
            r#"
            INSERT INTO collections (
                id,
                slug,
                title,
                supply,
                twitter,
                discord,
                website,
                verified,
                description,
                cover_url,
                royalty
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.id.clone());
            b.push_bind(item.slug.clone());
            b.push_bind(item.title.clone());
            b.push_bind(item.supply);
            b.push_bind(item.twitter.clone());
            b.push_bind(item.discord.clone());
            b.push_bind(item.website.clone());
            b.push_bind(item.verified);
            b.push_bind(item.description.clone());
            b.push_bind(item.cover_url.clone());
            b.push_bind(item.royalty.clone());
        })
        .push(
            r#"
            ON CONFLICT (id) DO UPDATE SET
                slug = EXCLUDED.slug,
                title = EXCLUDED.title,
                supply = EXCLUDED.supply,
                twitter = EXCLUDED.twitter,
                discord = EXCLUDED.discord,
                website = EXCLUDED.website,
                verified = EXCLUDED.verified,
                description = EXCLUDED.description,
                cover_url = EXCLUDED.cover_url,
                royalty = EXCLUDED.royalty
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert collections")?;

        Ok(res)
    }

    async fn fetch_collections(
        &self,
        query: &WhereCollectionSchema,
        order: Option<OrderCollectionSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<CollectionSchema>> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            SELECT
                c.id,
                c.slug, 
                c.supply, 
                c.title, 
                c.description, 
                c.cover_url, 
                c.verified,
                c.website,
                c.discord,
                c.twitter,
                c.royalty,
                cs.volumes::BIGINT           AS total_volume,
                cs.sales::BIGINT             AS total_sale,
                co.owners::BIGINT            AS total_owner,
                cl.floor_price::BIGINT       AS floor,
                cl.listed::BIGINT
            FROM collections c
                LEFT JOIN collection_sales cs ON cs.collection_id = c.id
                LEFT JOIN collection_listings cl ON cl.collection_id = c.id
                LEFT JOIN collection_owners co ON co.collection_id = c.id
            WHERE TRUE
            "#,
        );

        if let Some(collection_id) = query.collection_id.as_ref() {
            query_builder.push(" AND c.id = ");
            query_builder.push_bind(collection_id);
        }

        if let Some(wallet_address) = query.wallet_address.as_ref() {
            query_builder.push(
                r#"
                AND c.id IN (
                    SELECT DISTINCT n.collection_id FROM nfts n
                    WHERE n.owner = 
                "#,
            );
            query_builder.push_bind(wallet_address);
            query_builder.push(")");
        }

        if let Some(search) = query.search.as_ref() {
            query_builder.push(" AND c.title ILIKE ");
            query_builder.push_bind(search);
        }

        if let Some(order) = order.as_ref() {
            let mut order_builder = String::new();
            if let Some(order_type) = order.volume {
                order_builder.push_str(format!(" cs.volumes {},", order_type.to_string()).as_str());
            }

            if let Some(order_type) = order.floor {
                order_builder
                    .push_str(format!(" cl.floor_price {}", order_type.to_string()).as_str());
            }

            if let Some(order_type) = order.owners {
                order_builder.push_str(format!(" co.owners {},", order_type.to_string()).as_str());
            }

            if let Some(order_type) = order.market_cap {
                order_builder.push_str(
                    format!(" cl.floor_price * c.supply {},", order_type.to_string()).as_str(),
                );
            }

            if let Some(order_type) = order.sales {
                order_builder.push_str(format!(" cs.sales {},", order_type.to_string()).as_str());
            }

            if let Some(order_type) = order.listed {
                order_builder.push_str(format!(" cl.listed {},", order_type.to_string()).as_str());
            }

            let ordering = &order_builder[..(order_builder.len() - 1)];
            if ordering.trim().is_empty() {
                query_builder.push("ORDER BY cs.volume DESC ");
            } else {
                query_builder.push(format!("ORDER BY {}", ordering.to_lowercase().trim()));
            }
        }

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let res = query_builder
            .build_query_as::<CollectionSchema>()
            .fetch_all(&*self.pool)
            .await
            .context("Failed to fetch collections")?;

        Ok(res)
    }

    async fn fetch_trending(
        &self,
        query: &WhereTrendingSchema,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<TrendingSchema>> {
        let res = sqlx::query_as!(
            TrendingSchema,
            r#"
            WITH 
                nft_activities AS (
                    SELECT a.nft_id, COUNT(*) FROM activities a
                    WHERE a.tx_type IN ('mint', 'buy', 'transfer') AND a.collection_id = $1
                    GROUP BY a.nft_id
                ),
                price_activities AS (
                    SELECT DISTINCT ON (a.nft_id) a.nft_id, a,price FROM activities a
                    WHERE a.tx_type IN ('mint', 'buy', 'transfer') 
                        AND a.collection_id = $1
                        AND a.price > 0
                    ORDER BY a.nft_id, a.block_time DESC
                )
            SELECT 
                n.id                AS nft_id,
                n.collection_id     AS collection_id,
                na.count            AS tx_frequency,
                pa.price            AS last_price
            FROM nfts n
                LEFT JOIN nft_activities na ON na.nft_id = n.id
                LEFT JOIN price_activities pa ON na.nft_id = n.id
            ORDER BY na.count DESC
            LIMIT $2 OFFSET $3
            "#,
            query.collection_id,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection nft trendings")?;

        Ok(res)
    }

    async fn fetch_nft_changes(
        &self,
        query: &WhereNftChangeSchema,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<NftChangeSchema>> {
        let interval =
            string_utils::str_to_pginterval(&query.interval.clone().unwrap_or_default())?;
        let res = sqlx::query_as!(
            NftChangeSchema,
            r#"
            WITH 
                current_nft_owners AS (
                    SELECT n.owner, COUNT(*) FROM nfts n
                    WHERE n.burned IS NULL OR NOT n.burned AND n.collection_id = $1
                    GROUP BY n.collection_id, n.owner
                ),
                transfer_in AS (
                    SELECT a.collection_id, a.receiver AS address, COUNT(*) FROM activities a
                    WHERE ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL) 
                        AND a.tx_type IN ('transfer', 'buy')
                        AND a.collection_id = $1
                    GROUP BY a.collection_id, a.receiver
                ),
                transfer_out AS (
                    SELECT a.collection_id, a.sender AS address, COUNT(*) FROM activities a
                    WHERE ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL) 
                        AND a.tx_type IN ('transfer', 'buy')
                        AND a.collection_id = $1
                    GROUP BY a.collection_id, a.sender
                )
            SELECT 
                w.address, 
                (COALESCE(tout.count, 0) - COALESCE(tin.count, 0)) 	AS change,
                COALESCE(co.count, 0) 								AS quantity	
            FROM wallets w
                JOIN transfer_in tin ON tin.address = w.address
                JOIN transfer_out tout ON tout.address = w.address
                JOIN current_nft_owners co ON co.owner = w.address
            ORDER BY change DESC
            LIMIT $3 OFFSET $4
            "#,
            query.collection_id,
            interval,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection profit leaders")?;

        Ok(res)
    }

    async fn fetch_profit_leaderboards(
        &self,
        query: &WhereLeaderboardSchema,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ProfitLeaderboardSchema>> {
        let res = sqlx::query_as!(
            ProfitLeaderboardSchema,
            r#"
            WITH
                bought_activities AS (
                    SELECT a.collection_id, a.receiver AS address, COUNT(*) AS bought, SUM(price) AS price FROM activities a
                    WHERE a.tx_type = 'buy' AND a.collection_id = $1
                    GROUP BY a.collection_id, a.receiver 
                ),
                sold_activities AS (
                    SELECT a.collection_id, a.sender AS address, COUNT(*) AS sold, SUM(price) AS price FROM activities a
                    WHERE a.tx_type = 'buy' AND a.collection_id = $1
                    GROUP BY a.collection_id, a.sender
                )
            SELECT
                w.address,
                ba.bought, 
                sa.sold, 
                ba.price                                                                AS spent,
                (COALESCE(sa.price, 0) - COALESCE(ba.price, 0)) 	                    AS total_profit
            FROM wallets w
                JOIN bought_activities ba ON ba.address = w.address
                JOIN sold_activities sa ON sa.address = w.address
            LIMIT $2 OFFSET $3
            "#,
            query.collection_id,
            limit,
            offset,
        ).fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection profit leaders")?;

        Ok(res)
    }

    async fn fetch_attributes(&self, collection_id: &str) -> anyhow::Result<Vec<AttributeSchema>> {
        let res = sqlx::query_as!(
            AttributeSchema,
            r#"
            SELECT 
                ar.type                      AS attr_type, 
                jsonb_agg(DISTINCT ar.value) AS values 
            FROM attribute_rarities ar
            WHERE ar.collection_id = $1
            GROUP BY ar.collection_id, ar.type
            "#,
            collection_id,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch attributes")?;

        Ok(res)
    }

    async fn fetch_top_buyers(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<TopBuyerSchema>> {
        let res = sqlx::query_as!(
            TopBuyerSchema,
            r#"
            SELECT
                a.receiver      AS buyer, 
                COUNT(*)        AS bought, 
                SUM(a.price)    AS volume
            FROM activities a
            WHERE a.tx_type = 'buy'
                AND a.collection_id = $1
                AND ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL)
            GROUP BY a.collection_id, a.receiver
            ORDER BY bought DESC, volume DESC
            LIMIT 10
            "#,
            collection_id,
            interval,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection top buyers")?;

        Ok(res)
    }

    async fn fetch_top_sellers(
        &self,
        collection_id: &str,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<TopSellerSchema>> {
        let res = sqlx::query_as!(
            TopSellerSchema,
            r#"
            SELECT
                a.sender            AS seller, 
                COUNT(*)            AS sold, 
                SUM(a.price)        AS volume
            FROM activities a
            WHERE a.tx_type = 'buy'
                AND a.collection_id = $1
                AND ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL)
            GROUP BY a.collection_id, a.sender
            ORDER BY sold DESC, volume DESC
            LIMIT 10
            "#,
            collection_id,
            interval,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection top buyers")?;

        Ok(res)
    }

    async fn fetch_nft_holders(
        &self,
        collection_id: &str,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<NftHolderSchema>> {
        let res = sqlx::query_as!(
            NftHolderSchema,
            r#"
            WITH 
                mint_activities AS (
                    SELECT
                        a.receiver  AS address, 
                        COUNT(*)    AS count
                    FROM activities a
                    WHERE a.tx_type = 'mint' AND a.collection_id = $1
                    GROUP BY a.receiver
                ),
                send_activities AS (
                    SELECT
                        a.sender    AS address, 
                        COUNT(*)    AS count
                    FROM activities a
                    WHERE a.tx_type = 'buy' AND a.collection_id = $1
                    GROUP BY a.sender
                ),
                receive_activities AS (
                    SELECT
                        a.receiver  AS address, 
                        COUNT(*)    AS count
                    FROM activities a
                    WHERE a.tx_type = 'buy' AND a.collection_id = $1
                    GROUP BY a.receiver
                ),
                nft_owners AS (
                    SELECT 
                        n.owner     AS address,
                        COUNT(*)    AS count
                    FROM nfts n
                    WHERE n.collection_id = $1 AND (n.burned IS NULL OR NOT n.burned)
                    GROUP BY n.owner
                )
            SELECT 
                no.address, 
                no.count            AS quantity, 
                ma.count            AS mint,
                sa.count            AS send,
                ra.count            AS receive
            FROM nft_owners no
                LEFT JOIN mint_activities ma ON ma.address = no.address
                LEFT JOIN send_activities sa ON sa.address = no.address
                LEFT JOIN receive_activities ra ON ra.address = no.address
            ORDER BY no.count
            LIMIT $2 OFFSET $3
            "#,
            collection_id,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection nft holders")?;

        Ok(res)
    }

    async fn fetch_nft_amount_distribution(
        &self,
        collection_id: &str,
    ) -> anyhow::Result<NftAmountDistributionSchema> {
        let res = sqlx::query_as!(
            NftAmountDistributionSchema,
            r#"
            WITH nft_distributions AS (
                SELECT n.collection_id, n.owner, COUNT(*) FROM nfts n
                WHERE n.collection_id = $1
                GROUP BY n.collection_id, n.owner
            )
            SELECT 
                SUM(
                    CASE 
                        WHEN nd.count = 1 THEN 1
                        ELSE 0
                    END
                ) AS range_1,
                SUM(
                    CASE 
                        WHEN nd.count = 2 OR nd.count = 3 THEN 1
                        ELSE 0
                    END
                ) AS range_2_to_3,
                SUM(
                    CASE 
                        WHEN nd.count >= 4 AND nd.count <= 10 THEN 1
                        ELSE 0
                    END
                ) AS range_4_to_10,
                SUM(
                    CASE 
                        WHEN nd.count >= 11 AND nd.count <= 50 THEN 1
                        ELSE 0
                    END
                ) AS range_11_to_50,
                SUM(
                    CASE 
                        WHEN nd.count >= 50 AND nd.count <= 100 THEN 1
                        ELSE 0
                    END
                ) AS range_51_to_100,
                SUM(
                    CASE 
                        WHEN nd.count > 100 THEN 1
                        ELSE 0
                    END
                ) AS range_gt_100
            FROM nft_distributions nd
            GROUP BY nd.collection_id
            "#,
            collection_id
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch nft amount distribution")?;

        Ok(res)
    }

    async fn fetch_nft_period_distribution(
        &self,
        id: &str,
    ) -> anyhow::Result<NftPeriodDistributionSchema> {
        let res = sqlx::query_as!(
            NftPeriodDistributionSchema,
            r#"
            WITH
                nft_periods AS (
                	SELECT
                        ra.collection_id,
                        COALESCE(EXTRACT(EPOCH FROM sa.block_time), EXTRACT(EPOCH FROM ra.block_time)) 
                            - EXTRACT(EPOCH FROM ra.block_time) AS period 
                    FROM activities ra
                        LEFT JOIN activities sa ON ra.receiver = sa.sender AND ra.nft_id = sa.nft_id AND ra.collection_id = sa.collection_id
                    WHERE ra.receiver IS NOT NULL AND ra.collection_id = $1 AND ra.tx_type IN ('transfer', 'buy', 'mint')
                )
            SELECT
                SUM(
                    CASE 
                        WHEN np.period / EXTRACT(EPOCH FROM '1 day'::INTERVAL) < 1 THEN 1
                        ELSE 0
                    END
                ) AS range_lt_24h,
                SUM(
                    CASE 
                        WHEN np.period / EXTRACT(EPOCH FROM '1 day'::INTERVAL) >= 1 AND np.period / EXTRACT(EPOCH FROM '1 day'::INTERVAL) <= 7 THEN 1
                        ELSE 0
                    END
                ) AS range_1d_to_7d,
                SUM(
                    CASE 
                        WHEN np.period / EXTRACT(EPOCH FROM '1 day'::INTERVAL) > 7 AND np.period / EXTRACT(EPOCH FROM '1 day'::INTERVAL) <= 30 THEN 1
                        ELSE 0
                    END
                ) AS range_7d_to_30d,
                SUM(
                    CASE 
                        WHEN np.period / EXTRACT(EPOCH FROM '1 month'::INTERVAL) > 1 AND np.period / EXTRACT(EPOCH FROM '1 month'::INTERVAL) <= 3 THEN 1
                        ELSE 0
                    END
                ) AS range_30d_to_3m,
                SUM(
                    CASE 
                        WHEN np.period / EXTRACT(EPOCH FROM '1 month'::INTERVAL) > 3 AND np.period / EXTRACT(EPOCH FROM '1 year'::INTERVAL) <= 1 THEN 1
                        ELSE 0
                    END
                ) AS range_3m_to_1y,
                SUM(
                    CASE 
                        WHEN np.period / EXTRACT(EPOCH FROM '1 year'::INTERVAL) > 1 THEN 1
                        ELSE 0
                    END
                ) AS range_gte_1y
            FROM nft_periods np
            GROUP BY np.collection_id
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch nft period distribution")?;

        Ok(res)
    }

    async fn fetch_floor_charts(
        &self,
        collection_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        interval: PgInterval,
    ) -> anyhow::Result<Vec<DataPointSchema>> {
        let res = sqlx::query_as!(
            DataPointSchema,
            r#"
            WITH 
                time_series AS (
                    SELECT GENERATE_SERIES($2::TIMESTAMPTZ, $3::TIMESTAMPTZ, $4::INTERVAL) AS time_bin
                ),
                floor_prices AS (
                    SELECT 
                        ts.time_bin AS time,
                        COALESCE(
                            (
                                SELECT a.price FROM activities a
                                WHERE a.tx_type = 'list'
                                    AND a.collection_id = $1
                                    AND a.block_time >= ts.time_bin AND a.block_time < ts.time_bin + $4::INTERVAL
                                ORDER BY a.price ASC
                                LIMIT 1
                            ),
                            0
                        ) AS floor
                    FROM time_series ts
                    ORDER BY ts.time_bin
                )
            SELECT 
                ts.time_bin AS x,
                COALESCE(
                    (
                        SELECT fp.floor FROM floor_prices fp
                        WHERE fp.time <= ts.time_bin
                        LIMIT 1
                    ),
                    (
                        SELECT a.price FROM activities a
                        WHERE a.tx_type = 'list'
                            AND a.collection_id = $1
                            AND a.block_time <= ts.time_bin
                        ORDER BY a.price ASC
                        LIMIT 1
                    ),
                    0
                ) AS y
            FROM time_series ts
            ORDER BY ts.time_bin
            "#,
            collection_id,
            start_time,
            end_time,
            interval,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection floor chart")?;

        Ok(res)
    }
}
