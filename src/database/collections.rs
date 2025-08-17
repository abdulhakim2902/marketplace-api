use std::{str::FromStr, sync::Arc};

use crate::{
    models::{
        db::collection::DbCollection,
        schema::{
            collection::{
                CollectionSchema, FilterCollectionSchema, PeriodType,
                attribute::CollectionAttributeSchema,
                nft_change::{FilterNftChangeSchema, NftChangeSchema},
                nft_distribution::{NftAmountDistributionSchema, NftPeriodDistributionSchema},
                nft_holder::NftHolderSchema,
                profit_leaderboard::ProfitLeaderboardSchema,
                stat::CollectionStatSchema,
                top_wallet::{FilterTopWalletSchema, TopWalletSchema, TopWalletType},
                trending::{CollectionTrendingSchema, OrderTrendingType},
                trending_nft::TrendingNftSchema,
            },
            data_point::{DataPointSchema, FilterFloorChartSchema},
        },
    },
    utils::string_utils,
};
use anyhow::Context;
use chrono::DateTime;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait ICollections: Send + Sync {
    async fn tx_insert_collections(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbCollection>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_collections(
        &self,
        filter: FilterCollectionSchema,
    ) -> anyhow::Result<Vec<CollectionSchema>>;

    async fn fetch_trendings(
        &self,
        interval: &str,
        limit: i64,
        offset: i64,
        order: OrderTrendingType,
    ) -> anyhow::Result<Vec<CollectionTrendingSchema>>;

    async fn fetch_stats(&self, collection_id: Uuid) -> anyhow::Result<CollectionStatSchema>;

    async fn fetch_trending_nfts(
        &self,
        collection_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<TrendingNftSchema>>;

    async fn fetch_nft_changes(
        &self,
        filter: FilterNftChangeSchema,
    ) -> anyhow::Result<Vec<NftChangeSchema>>;

    async fn fetch_profit_leaderboards(
        &self,
        collection_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ProfitLeaderboardSchema>>;

    async fn fetch_attributes(
        &self,
        collection_id: Uuid,
    ) -> anyhow::Result<Vec<CollectionAttributeSchema>>;

    async fn fetch_top_wallets(
        &self,
        filter: FilterTopWalletSchema,
    ) -> anyhow::Result<Vec<TopWalletSchema>>;

    async fn fetch_nft_holders(
        &self,
        collection_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<NftHolderSchema>>;

    async fn fetch_nft_amount_distribution(
        &self,
        collection_id: Uuid,
    ) -> anyhow::Result<NftAmountDistributionSchema>;

    async fn fetch_nft_period_distribution(
        &self,
        collection_id: Uuid,
    ) -> anyhow::Result<NftPeriodDistributionSchema>;

    async fn fetch_floor_charts(
        &self,
        filter: FilterFloorChartSchema,
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
        filter: FilterCollectionSchema,
    ) -> anyhow::Result<Vec<CollectionSchema>> {
        let query = filter.where_.unwrap_or_default();
        let order = filter.order_by;
        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        let mut query_builder = QueryBuilder::<Postgres>::new("");

        if let Some(period) = query.periods {
            query_builder.push(
                r#"
                WITH sale_activities AS (
                    SELECT
                        activities.collection_id,
                        SUM(activities.price)::BIGINT   AS volume,
                        SUM(activities.usd_price)       AS volume_usd,
                        COUNT(*)                        AS sales
                    FROM activities
                "#,
            );

            match period {
                PeriodType::Hours1 => {
                    query_builder.push(
                        r#"
                        WHERE activities.block_time >= NOW() - '1h'::INTERVAL
                        "#,
                    );
                }
                PeriodType::Hours6 => {
                    query_builder.push(
                        r#"
                        WHERE activities.block_time >= NOW() - '6h'::INTERVAL
                        "#,
                    );
                }
                PeriodType::Days1 => {
                    query_builder.push(
                        r#"
                        WHERE activities.block_time >= NOW() - '24h'::INTERVAL
                        "#,
                    );
                }
                PeriodType::Days7 => {
                    query_builder.push(
                        r#"
                        WHERE activities.block_time >= NOW() - '7d'::INTERVAL
                        "#,
                    );
                }
            }

            query_builder.push(" GROUP BY activities.collection_id)");
        } else {
            query_builder.push(
                r#"
                WITH sale_activities AS (
                    SELECT
                        NULL::UUID                      AS collection_id,
                        NULL::BIGINT                    AS volume,
                        NULL::NUMERIC                   AS volume_usd,
                        NULL::BIGINT                    AS sales
                )
                "#,
            );
        }

        query_builder.push(
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
                c.floor,
                c.owners,
                COALESCE(sa.volume, c.volume)               AS volume,
                COALESCE(sa.volume_usd, c.volume_usd)       AS volume_usd,
                COALESCE(sa.sales, c.sales)                 AS sales,
                c.listed
            FROM collections c
                LEFT JOIN sale_activities sa ON sa.collection_id = c.id
            WHERE TRUE
            "#,
        );

        if let Some(collection_id) = query.collection_id.as_ref() {
            let collection_id = Uuid::from_str(collection_id).ok();
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
                order_builder
                    .push_str(format!(" volume {} NULLS LAST,", order_type.to_string()).as_str());
            }

            if let Some(order_type) = order.floor {
                order_builder
                    .push_str(format!(" c.floor {} NULLS LAST,", order_type.to_string()).as_str());
            }

            if let Some(order_type) = order.owners {
                order_builder
                    .push_str(format!(" c.owners {} NULLS LAST,", order_type.to_string()).as_str());
            }

            if let Some(order_type) = order.market_cap {
                order_builder.push_str(
                    format!(" c.floor * c.supply {} NULLS LAST,", order_type.to_string()).as_str(),
                );
            }

            if let Some(order_type) = order.sales {
                order_builder
                    .push_str(format!(" sales {} NULLS LAST,", order_type.to_string()).as_str());
            }

            if let Some(order_type) = order.listed {
                order_builder
                    .push_str(format!(" c.listed {} NULLS LAST,", order_type.to_string()).as_str());
            }

            let ordering = &order_builder[..(order_builder.len() - 1)];
            if ordering.trim().is_empty() {
                query_builder.push(" ORDER BY c.volume DESC NULLS LAST ");
            } else {
                query_builder.push(format!(" ORDER BY {}", ordering.to_lowercase().trim()));
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

    async fn fetch_trendings(
        &self,
        interval: &str,
        limit: i64,
        offset: i64,
        order: OrderTrendingType,
    ) -> anyhow::Result<Vec<CollectionTrendingSchema>> {
        let interval = string_utils::str_to_pginterval(interval).ok().flatten();
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            WITH 
                sale_activities AS (
                    SELECT
                        collection_id,
                        SUM(price)::BIGINT              AS volume,
                        SUM(usd_price)                  AS volume_usd,
                        COUNT(*)                        AS sales
                    FROM activities
                    WHERE tx_type IN ('buy', 'accept-bid', 'accept-collection-bid')
                        AND (
            "#,
        );

        query_builder.push_bind(interval);
        query_builder.push("::INTERVAL IS NULL OR block_time >= NOW() - ");
        query_builder.push_bind(interval);
        query_builder.push("::INTERVAL)");
        query_builder.push(" GROUP BY collection_id),");

        query_builder.push(
            r#"
                collection_trendings AS (
                    SELECT 
                        c.id,
                        c.floor,
                        c.owners,
                        c.listed,
                        c.supply,
                        sa.volume,
                        sa.volume_usd,
                        sa.sales,
                        c.floor * c.supply      AS market_cap
                    FROM collections c
                        LEFT JOIN sale_activities sa ON sa.collection_id = c.id
                )
            SELECT * FROM collection_trendings
            "#,
        );

        match order {
            OrderTrendingType::Volume => {
                query_builder.push("ORDER BY volume DESC NULLS LAST");
            }
            OrderTrendingType::Floor => {
                query_builder.push("ORDER BY floor DESC NULLS LAST");
            }
            OrderTrendingType::Listed => {
                query_builder.push("ORDER BY listed DESC NULLS LAST");
            }
            OrderTrendingType::MarketCap => {
                query_builder.push("ORDER BY market_cap DESC NULLS LAST");
            }
            OrderTrendingType::Owners => {
                query_builder.push("ORDER BY owners DESC NULLS LAST");
            }
            OrderTrendingType::Sales => {
                query_builder.push("ORDER BY sales DESC NULLS LAST");
            }
        }

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let res = query_builder
            .build_query_as::<CollectionTrendingSchema>()
            .fetch_all(&*self.pool)
            .await
            .context("Failed to fetch collection trendings")?;

        Ok(res)
    }

    async fn fetch_stats(&self, collection_id: Uuid) -> anyhow::Result<CollectionStatSchema> {
        let res = sqlx::query_as!(
            CollectionStatSchema,
            r#"
            WITH
                top_bids AS (
                    SELECT
                        b.collection_id,
                        MAX(b.price)                AS price
                    FROM bids b
                    WHERE b.collection_id = $1
                        AND b.status = 'active'
                        AND b.bid_type = 'solo'
                        AND b.expired_at > NOW()
                    GROUP BY b.collection_id
                ),
                sale_activities AS (
                    SELECT
                        activities.collection_id,
                        SUM(activities.price)::BIGINT   AS volume,
                        COUNT(*)                        AS sales
                    FROM activities
                    WHERE activities.block_time >= NOW() - '24h'::INTERVAL
                        AND activities.tx_type IN ('buy', 'accept-bid', 'accept-collection-bid')
                        AND activities.collection_id = $1
                    GROUP BY activities.collection_id
                ),
                collection_scores AS (
                    SELECT DISTINCT ON (ca.collection_id, ca.type, ca.value)
                        ca.collection_id,
                        SUM(ca.score) AS score
                    FROM attributes ca
                    WHERE ca.collection_id = $1
                    GROUP BY ca.collection_id, ca.type, ca.value
                )
            SELECT
                c.floor,
                c.owners,
                c.listed,
                c.supply,
                c.volume                    AS total_volume,
                c.volume_usd                AS total_usd_volume,
                c.sales                     AS total_sales,
                sa.sales                    AS day_sales,
                sa.volume                   AS day_volume,
                tb.price                    AS top_offer,
                (1 / cs.score)::NUMERIC     AS rarity
            FROM collections c
                LEFT JOIN top_bids tb ON tb.collection_id = c.id
                LEFT JOIN sale_activities sa ON sa.collection_id = c.id
                LEFT JOIN collection_scores cs ON cs.collection_id = c.id
            WHERE c.id = $1
            "#,
            collection_id,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch collection stat")?;

        Ok(res)
    }

    async fn fetch_trending_nfts(
        &self,
        collection_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<TrendingNftSchema>> {
        let res = sqlx::query_as!(
            TrendingNftSchema,
            r#"
            WITH 
                nft_activities AS (
                    SELECT a.nft_id, COUNT(*) FROM activities a
                    WHERE a.tx_type IN ('mint', 'buy', 'transfer', 'accept-bid', 'accept-collection-bid') AND a.collection_id = $1
                    GROUP BY a.nft_id
                ),
                price_activities AS (
                    SELECT DISTINCT ON (a.nft_id) a.nft_id, a,price FROM activities a
                    WHERE a.tx_type IN ('mint', 'buy', 'transfer', 'accept-bid', 'accept-collection-bid') 
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
            collection_id,
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
        filter: FilterNftChangeSchema,
    ) -> anyhow::Result<Vec<NftChangeSchema>> {
        let query = filter.where_;

        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        let collection_id = Uuid::from_str(query.collection_id.as_str()).ok();

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
                        AND a.tx_type IN ('transfer', 'buy', 'accept-bid', 'accept-collection-bid')
                        AND a.collection_id = $1
                    GROUP BY a.collection_id, a.receiver
                ),
                transfer_out AS (
                    SELECT a.collection_id, a.sender AS address, COUNT(*) FROM activities a
                    WHERE ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL) 
                        AND a.tx_type IN ('transfer', 'buy', 'accept-bid', 'accept-collection-bid')
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
            collection_id,
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
        collection_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<ProfitLeaderboardSchema>> {
        let res = sqlx::query_as!(
            ProfitLeaderboardSchema,
            r#"
            WITH
                bought_activities AS (
                    SELECT a.collection_id, a.receiver AS address, COUNT(*) AS bought, SUM(price) AS price FROM activities a
                    WHERE a.tx_type IN ('buy', 'accept-bid', 'accept-collection-bid') AND a.collection_id = $1
                    GROUP BY a.collection_id, a.receiver 
                ),
                sold_activities AS (
                    SELECT a.collection_id, a.sender AS address, COUNT(*) AS sold, SUM(price) AS price FROM activities a
                    WHERE a.tx_type IN ('buy', 'accept-bid', 'accept-collection-bid') AND a.collection_id = $1
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
            collection_id,
            limit,
            offset,
        ).fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection profit leaders")?;

        Ok(res)
    }

    async fn fetch_attributes(
        &self,
        collection_id: Uuid,
    ) -> anyhow::Result<Vec<CollectionAttributeSchema>> {
        let res = sqlx::query_as!(
            CollectionAttributeSchema,
            r#"
            SELECT 
                ca.type                      AS type_,
                jsonb_agg(DISTINCT ca.value) AS values
            FROM attributes ca
            WHERE ca.collection_id = $1
            GROUP BY ca.collection_id, ca.type
            "#,
            collection_id,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch attributes")?;

        Ok(res)
    }

    async fn fetch_top_wallets(
        &self,
        filter: FilterTopWalletSchema,
    ) -> anyhow::Result<Vec<TopWalletSchema>> {
        let query = filter.where_;

        let type_ = query.type_;
        let collection_id = Uuid::from_str(query.collection_id.as_str()).ok();
        let interval = string_utils::str_to_pginterval(&query.interval.unwrap_or_default())
            .expect("Invalid interval");

        let limit = filter.limit.unwrap_or(10);

        let res = match type_ {
            TopWalletType::Buyer => sqlx::query_as!(
                TopWalletSchema,
                r#"
                SELECT
                    a.receiver      AS address,
                    COUNT(*)        AS total,
                    SUM(a.price)    AS volume
                FROM activities a
                WHERE a.tx_type IN ('buy', 'accept-bid', 'accept-collection-bid')
                    AND a.collection_id = $1
                    AND ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL)
                GROUP BY a.collection_id, a.receiver
                ORDER BY total DESC, volume DESC
                LIMIT $3
                "#,
                collection_id,
                interval,
                limit,
            )
            .fetch_all(&*self.pool)
            .await
            .context("Failed to fetch collection top buyers"),
            TopWalletType::Seller => sqlx::query_as!(
                TopWalletSchema,
                r#"
                SELECT
                    a.sender            AS address,
                    COUNT(*)            AS total,
                    SUM(a.price)        AS volume
                FROM activities a
                WHERE a.tx_type IN ('buy', 'accept-bid', 'accept-collection-bid')
                    AND a.collection_id = $1
                    AND ($2::INTERVAL IS NULL OR a.block_time >= NOW() - $2::INTERVAL)
                GROUP BY a.collection_id, a.sender
                ORDER BY total DESC, volume DESC
                LIMIT $3
                "#,
                collection_id,
                interval,
                limit,
            )
            .fetch_all(&*self.pool)
            .await
            .context("Failed to fetch collection top sellers"),
        }?;

        Ok(res)
    }

    async fn fetch_nft_holders(
        &self,
        collection_id: Uuid,
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
                    WHERE a.tx_type IN ('buy', 'accept-bid', 'accept-collection-bid') AND a.collection_id = $1
                    GROUP BY a.sender
                ),
                receive_activities AS (
                    SELECT
                        a.receiver  AS address, 
                        COUNT(*)    AS count
                    FROM activities a
                    WHERE a.tx_type IN ('buy', 'accept-bid', 'accept-collection-bid') AND a.collection_id = $1
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
        collection_id: Uuid,
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
        collection_id: Uuid,
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
                    WHERE ra.receiver IS NOT NULL AND ra.collection_id = $1 AND ra.tx_type IN ('transfer', 'buy', 'mint', 'accept-bid', 'accept-collection-bid')
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
            collection_id
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch nft period distribution")?;

        Ok(res)
    }

    async fn fetch_floor_charts(
        &self,
        filter: FilterFloorChartSchema,
    ) -> anyhow::Result<Vec<DataPointSchema>> {
        let query = filter.where_;

        let collection_id = Uuid::from_str(query.collection_id.as_str()).ok();

        let interval = string_utils::str_to_pginterval(query.interval.as_str())
            .expect("Invalid interval")
            .expect("Invalid interval");

        let start_date =
            DateTime::from_timestamp_millis(query.start_time).expect("Invalid start time");
        let end_date = DateTime::from_timestamp_millis(query.end_time).expect("Invalid end time");

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
            start_date,
            end_date,
            interval,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection floor chart")?;

        Ok(res)
    }
}
