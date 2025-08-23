use std::{collections::HashMap, sync::Arc};

use crate::{
    database::Schema,
    models::{
        db::collection::DbCollection,
        schema::{
            AggregateFieldsSchema, CoinType,
            collection::{
                AggregateCollectionFieldsSchema, CollectionSchema, DistinctCollectionSchema,
                OrderCollectionSchema, QueryCollectionSchema,
                attribute::CollectionAttributeSchema,
                holder::{CollectionHolderSchema, OrderHolderType},
                nft_change::NftChangeSchema,
                nft_distribution::{NftAmountDistributionSchema, NftPeriodDistributionSchema},
                nft_holder::NftHolderSchema,
                profit_leaderboard::ProfitLeaderboardSchema,
                stat::CollectionStatSchema,
                top_wallet::{TopWalletSchema, TopWalletType},
                trending::{CollectionTrendingSchema, OrderTrendingType},
                trending_nft::TrendingNftSchema,
            },
            data_point::DataPointSchema,
        },
    },
    utils::schema::{create_aggregate_query_builder, create_query_builder},
};
use anyhow::Context;
use async_graphql::{FieldError, dataloader::Loader};
use chrono::{DateTime, Utc};
use sqlx::{
    PgPool, Postgres, QueryBuilder, Transaction,
    postgres::{PgQueryResult, types::PgInterval},
};
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
        query: &QueryCollectionSchema,
        order: &OrderCollectionSchema,
        distinct: Option<&DistinctCollectionSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<CollectionSchema>>;

    async fn fetch_aggregate_collections(
        &self,
        selection: &HashMap<String, Vec<String>>,
        query: &QueryCollectionSchema,
        distinct: Option<&DistinctCollectionSchema>,
    ) -> anyhow::Result<AggregateFieldsSchema<AggregateCollectionFieldsSchema>>;

    async fn fetch_trendings(
        &self,
        limit: i64,
        offset: i64,
        order: OrderTrendingType,
        interval: Option<PgInterval>,
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
        collection_id: Uuid,
        limit: i64,
        offset: i64,
        interval: Option<PgInterval>,
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
        collection_id: Uuid,
        type_: TopWalletType,
        limit: i64,
        interval: Option<PgInterval>,
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

    async fn fetch_holders(
        &self,
        collection_id: Uuid,
        order: OrderHolderType,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<CollectionHolderSchema>>;

    async fn fetch_floor_charts(
        &self,
        collection_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        interval: PgInterval,
    ) -> anyhow::Result<Vec<DataPointSchema>>;

    async fn fetch_volume_charts(
        &self,
        collection_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        interval: PgInterval,
        coin_type: CoinType,
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
                royalty,
                table_handle,
                creator_address
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
            b.push_bind(item.table_handle.clone());
            b.push_bind(item.creator_address.clone());
        })
        .push(
            r#"
            ON CONFLICT (id) DO UPDATE SET
                slug = EXCLUDED.slug,
                title = COALESCE(EXCLUDED.title, collections.title),
                supply = COALESCE(EXCLUDED.supply, collections.supply),
                twitter = COALESCE(EXCLUDED.twitter, collections.twitter),
                discord = COALESCE(EXCLUDED.discord, collections.discord),
                website = COALESCE(EXCLUDED.website, collections.website),
                verified = COALESCE(EXCLUDED.verified, collections.verified),
                description = COALESCE(EXCLUDED.description, collections.description),
                cover_url = COALESCE(EXCLUDED.cover_url, collections.cover_url),
                royalty = COALESCE(EXCLUDED.royalty, collections.royalty),
                table_handle = COALESCE(EXCLUDED.table_handle, collections.table_handle),
                creator_address = COALESCE(EXCLUDED.creator_address, collections.creator_address)
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
        query: &QueryCollectionSchema,
        order: &OrderCollectionSchema,
        distinct: Option<&DistinctCollectionSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<CollectionSchema>> {
        create_query_builder(
            "collections",
            Schema::Collections,
            query,
            order,
            distinct,
            limit,
            offset,
        )
        .build_query_as::<CollectionSchema>()
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collections")
    }

    async fn fetch_aggregate_collections(
        &self,
        selection: &HashMap<String, Vec<String>>,
        query: &QueryCollectionSchema,
        distinct: Option<&DistinctCollectionSchema>,
    ) -> anyhow::Result<AggregateFieldsSchema<AggregateCollectionFieldsSchema>> {
        if selection.is_empty() {
            return Ok(AggregateFieldsSchema::default());
        }

        let table = if let Some(distinct) = distinct {
            format!("(SELECT DISTINCT ON ({}) * FROM collections)", distinct)
        } else {
            "(SELECT * FROM collections)".to_string()
        };

        let value =
            create_aggregate_query_builder(table.as_str(), selection, Schema::Collections, query)
                .build_query_scalar::<serde_json::Value>()
                .fetch_one(&*self.pool)
                .await
                .context("Failed to fetch aggregate collections")?;

        let result =
            serde_json::from_value::<AggregateFieldsSchema<AggregateCollectionFieldsSchema>>(value)
                .context("Failed to parse aggregate result")?;

        Ok(result)
    }

    async fn fetch_trendings(
        &self,
        limit: i64,
        offset: i64,
        order: OrderTrendingType,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<CollectionTrendingSchema>> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            WITH 
                nft_owners AS (
                    SELECT collection_id, COUNT(DISTINCT owner) AS total
                    FROM nfts
                    WHERE burned IS NULL OR NOT burned
                    GROUP BY collection_id 
                ),
                listings AS (
                    SELECT collection_id, COUNT(*) total
                    FROM listings
                    WHERE listed
                    GROUP BY collection_id
                ),
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
                previous_sale_activities AS (
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
        query_builder.push("::INTERVAL IS NULL OR (block_time >= NOW() - '24h'::INTERVAL - ");
        query_builder.push_bind(interval);
        query_builder.push("::INTERVAL AND block_time < NOW() - ");
        query_builder.push_bind(interval);
        query_builder.push("::INTERVAL))");
        query_builder.push(" GROUP BY collection_id),");

        query_builder.push(
            r#"
                collection_trendings AS (
                    SELECT 
                        c.id                    AS collection_id,
                        sa.volume               AS current_volume,
                        sa.volume_usd           AS current_usd_volume,
                        sa.sales                AS current_trades_count,
                        psa.volume              AS previous_volume,
                        psa.volume_usd          AS previous_usd_volume,
                        psa.sales               AS previous_trades_count,
                        no.total                AS owners,
                        l.total                 AS listed,
                        c.supply,
                        c.floor
                    FROM collections c
                        LEFT JOIN sale_activities sa ON sa.collection_id = c.id
                        LEFT JOIN previous_sale_activities psa ON psa.collection_id = c.id
                        LEFT JOIN nft_owners no ON no.collection_id = c.id
                        LEFT JOIN listings l ON l.collection_id = c.id
                )
            SELECT * FROM collection_trendings
            "#,
        );

        match order {
            OrderTrendingType::Volume => {
                query_builder.push("ORDER BY current_volume DESC NULLS LAST");
            }
            OrderTrendingType::Floor => {
                query_builder.push("ORDER BY floor DESC NULLS LAST");
            }
            OrderTrendingType::Listed => {
                query_builder.push("ORDER BY listed DESC NULLS LAST");
            }
            OrderTrendingType::MarketCap => {
                query_builder.push("ORDER BY floor * supply DESC NULLS LAST");
            }
            OrderTrendingType::Owners => {
                query_builder.push("ORDER BY owners DESC NULLS LAST");
            }
            OrderTrendingType::Sales => {
                query_builder.push("ORDER BY current_trades_count DESC NULLS LAST");
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
                total_sale_activities AS (
                    SELECT
                        activities.collection_id,
                        SUM(activities.price)::BIGINT   AS volume,
                        COUNT(*)                        AS sales
                    FROM activities
                    WHERE activities.tx_type IN ('buy', 'accept-bid', 'accept-collection-bid')
                        AND activities.collection_id = $1
                    GROUP BY activities.collection_id
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
                collection_listings AS (
                    SELECT
                        l.collection_id,
                        COUNT(*) AS listed
                    FROM listings l
                    WHERE l.collection_id = $1 AND l.listed
                    GROUP BY l.collection_id
                ),
                collection_owners AS (
                    SELECT collection_id, COUNT(DISTINCT owner) FROM nfts
                    WHERE collection_id = $1 
                        AND (burned IS NULL OR NOT burned)
                    GROUP BY collection_id
                ),
                collection_scores AS (
                    SELECT DISTINCT ON (ca.collection_id, ca.attr_type, ca.value)
                        ca.collection_id,
                        SUM(ca.score) AS score
                    FROM attributes ca
                    WHERE ca.collection_id = $1
                    GROUP BY ca.collection_id, ca.attr_type, ca.value
                )
            SELECT
                c.floor,
                co.count                    AS owners,
                cl.listed,
                c.supply,
                c.volume                    AS total_volume,
                c.volume_usd                AS total_usd_volume,
                tsa.sales                   AS total_sales,
                sa.sales                    AS day_sales,
                sa.volume                   AS day_volume,
                tb.price                    AS top_offer,
                (1 / cs.score)::NUMERIC     AS rarity
            FROM collections c
                LEFT JOIN top_bids tb ON tb.collection_id = c.id
                LEFT JOIN sale_activities sa ON sa.collection_id = c.id
                LEFT JOIN total_sale_activities tsa ON tsa.collection_id = c.id
                LEFT JOIN collection_scores cs ON cs.collection_id = c.id
                LEFT JOIN collection_listings cl ON cl.collection_id = c.id
                LEFT JOIN collection_owners co ON co.collection_id = c.id
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
        collection_id: Uuid,
        limit: i64,
        offset: i64,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<NftChangeSchema>> {
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
            WITH collection_attributes AS (
                SELECT DISTINCT ON(collection_id, attr_type, value)
                    collection_id,
                    attr_type,
                    value,
                    rarity,
                    score
                FROM attributes
                WHERE collection_id = $1
            )
            SELECT 
                ca.attr_type,
                jsonb_agg(json_build_object('value', ca.value, 'rarity', ca.rarity, 'score', ca.score)) as json
            FROM collection_attributes ca
            GROUP BY ca.attr_type
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
        collection_id: Uuid,
        type_: TopWalletType,
        limit: i64,
        interval: Option<PgInterval>,
    ) -> anyhow::Result<Vec<TopWalletSchema>> {
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

    async fn fetch_holders(
        &self,
        collection_id: Uuid,
        order: OrderHolderType,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<CollectionHolderSchema>> {
        let res = sqlx::query_as!(
            CollectionHolderSchema,
            r#"
            WITH 
                current_holders AS (
                    SELECT collection_id, owner, COUNT(*) AS count
                    FROM nfts
                    WHERE collection_id = $1 AND (burned IS NULL OR NOT burned)
                    GROUP BY collection_id, owner
                ),
                sale_activities AS (
                    SELECT
                        collection_id, 
                        sender, 
                        COUNT(*) AS sales, 
                        SUM(price) AS volume
                    FROM activities
                    WHERE tx_type IN ('buy', 'accept-bid', 'accept-collection-bid')
                        AND collection_id = $1
                    GROUP BY collection_id, sender
                ),
                transfer_activities AS (
                    SELECT
                        collection_id, 
                        receiver, 
                        COUNT(DISTINCT nft_id) AS transfers
                    FROM activities
                    WHERE tx_type IN ('mint', 'transfer', 'buy', 'accept-bid', 'accept-collection-bid')
                        AND collection_id = $1
                    GROUP BY collection_id, receiver
                ),
                owner_holding_time AS (
                	SELECT
                        ra.collection_id,
                        ra.receiver,
                        COUNT(*),
                        SUM(COALESCE(EXTRACT(EPOCH FROM sa.block_time), EXTRACT(EPOCH FROM ra.block_time)) 
                            - EXTRACT(EPOCH FROM ra.block_time)) AS holding_time 
                    FROM activities ra
                        LEFT JOIN activities sa ON ra.receiver = sa.sender AND ra.nft_id = sa.nft_id AND ra.collection_id = sa.collection_id
                    WHERE ra.receiver IS NOT NULL AND ra.collection_id = $1 AND ra.tx_type IN ('transfer', 'buy', 'mint', 'accept-bid', 'accept-collection-bid')
                    GROUP BY ra.collection_id, ra.receiver
                )
            SELECT 
                id                  AS collection_id,
                ch.count            AS current_holdings,
                ch.owner,
                sa.sales            AS sold,
                sa.volume           AS sold_volume,
                ta.transfers        AS total_holdings,
                oht.holding_time	AS total_holding_time
            FROM collections
                LEFT JOIN current_holders ch ON ch.collection_id = collections.id
                LEFT JOIN sale_activities sa ON sa.collection_id = collections.id AND sa.sender = ch.owner
                LEFT JOIN transfer_activities ta ON ta.collection_id = collections.id AND ta.receiver = ch.owner
                LEFT JOIN owner_holding_time oht ON oht.collection_id = collections.id AND oht.receiver = ch.owner
            WHERE id = $1
            ORDER BY 
                CASE $2
                    WHEN 'curent_holdings' THEN ch.count
                    WHEN 'sold' THEN sa.sales
                    WHEN 'average_hold' THEN oht.holding_time / NULLIF(oht.count, 0)
                    WHEN 'average_sold' THEN sa.volume / NULLIF(sa.sales, 0)
                END DESC
            LIMIT $3 OFFSET $4
            "#,
            collection_id,
            order.to_string(),
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection holders")?;

        Ok(res)
    }

    async fn fetch_floor_charts(
        &self,
        collection_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
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
            start_date,
            end_date,
            interval,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection floor chart")?;

        Ok(res)
    }

    async fn fetch_volume_charts(
        &self,
        collection_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        interval: PgInterval,
        coin_type: CoinType,
    ) -> anyhow::Result<Vec<DataPointSchema>> {
        let res = sqlx::query_as!(
            DataPointSchema,
            r#"
            WITH 
                time_series AS (
                    SELECT GENERATE_SERIES($2::TIMESTAMPTZ, $3::TIMESTAMPTZ, $4::INTERVAL) AS time_bin
                ),
                collection_volumes AS (
                    SELECT * FROM activities
                    WHERE tx_type IN ('buy', 'accept-bid', 'accept-collection-bid', 'mint', 'transfer')
                        AND collection_id = $1
                        AND block_time BETWEEN $2 AND $3
                        AND price > 0
                )
            SELECT 
                ts.time_bin                             AS x,
                COALESCE(SUM(
                    CASE 
                        WHEN $5 = 'apt' THEN cv.price
                        WHEN $5 = 'usd' THEN cv.usd_price
                        ELSE 0
                    END
                ), 0)::BIGINT      AS y
            FROM time_series ts
                LEFT JOIN collection_volumes cv ON cv.block_time >= ts.time_bin AND cv.block_time < ts.time_bin + $4::INTERVAL
            GROUP BY ts.time_bin
            ORDER BY ts.time_bin
            "#,
            collection_id,
            start_date,
            end_date,
            interval,
            coin_type.to_string()
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch collection volume chart")?;

        Ok(res)
    }
}

impl Loader<Uuid> for Collections {
    type Value = CollectionSchema;
    type Error = FieldError;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let res = sqlx::query_as!(
            CollectionSchema,
            r#"
            SELECT * FROM collections
            WHERE id = ANY($1)
            "#,
            keys
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(res.into_iter().map(|c| (c.id, c)).collect())
    }
}
