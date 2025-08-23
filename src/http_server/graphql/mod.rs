pub mod guard;
pub mod http;

use std::sync::Arc;

use crate::{
    database::{
        Database, IDatabase, activities::IActivities, attributes::IAttributes, bids::IBids,
        collections::ICollections, listings::IListings, marketplaces::IMarketplaces, nfts::INfts,
        wallets::IWallets,
    },
    http_server::graphql::{guard::UserGuard, http::graphiql_v2_source::GraphiQLSource},
    models::schema::{
        AggregateSchema, CoinType,
        activity::{
            ActivitySchema, AggregateActivitySchema, DistinctActivitySchema, OrderActivitySchema,
            QueryActivitySchema, profit_loss::ProfitLossSchema,
        },
        attribute::{
            AggregateAttributeSchema, AttributeSchema, DistinctAttributeSchema,
            OrderAttributeSchema, QueryAttributeSchema,
        },
        bid::{AggregateBidSchema, BidSchema, DistinctBidSchema, OrderBidSchema, QueryBidSchema},
        collection::{
            AggregateCollectionSchema, CollectionSchema, DistinctCollectionSchema,
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
        data_point::{DataPointSchema, validate_data_set},
        get_aggregate_selection,
        listing::{
            AggregateListingSchema, DistinctListingSchema, ListingSchema, OrderListingSchema,
            QueryListingSchema,
        },
        marketplace::MarketplaceSchema,
        nft::{AggregateNftSchema, DistinctNftSchema, NftSchema, OrderNftSchema, QueryNftSchema},
        wallet::{nft_holding_period::NftHoldingPeriodSchema, stats::StatsSchema},
    },
    utils::string_utils,
};
use async_graphql::{
    Context, FieldError, FieldResult, InputValueError, InputValueResult, Object, Scalar,
    ScalarType, Value,
};
use axum::response::{Html, IntoResponse};
use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgInterval;
use uuid::Uuid;

pub async fn graphql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .title("NFT Aggregator GraphQL API")
            .endpoint("/")
            .finish(),
    )
}

#[derive(Debug, Clone)]
pub struct Wrapper<T>(pub T);

#[Scalar]
impl ScalarType for Wrapper<PgInterval> {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(s) = &value {
            let interval = string_utils::str_to_pginterval(&s)
                .map_err(|_| InputValueError::expected_type(value))?;
            Ok(Wrapper(interval))
        } else {
            Err(InputValueError::custom("Invalid PgInterval format"))
        }
    }

    fn to_value(&self) -> Value {
        let interval = &self.0;
        Value::Number(interval.microseconds.into())
    }
}

#[Scalar]
impl ScalarType for Wrapper<DateTime<Utc>> {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(s) = &value {
            let dt = DateTime::parse_from_rfc3339(s)
                .map_err(|_| InputValueError::expected_type(value))?;
            Ok(Wrapper(dt.with_timezone(&Utc)))
        } else if let Value::Number(n) = &value {
            n.as_i64()
                .and_then(|ts| DateTime::from_timestamp_millis(ts))
                .map(|dt| Wrapper(dt.with_timezone(&Utc)))
                .ok_or_else(|| InputValueError::expected_type(value))
        } else {
            Err(InputValueError::custom("Invalid DateTime format"))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_rfc3339())
    }
}

pub struct Query;

#[Object]
impl Query {
    #[graphql(guard = "UserGuard")]
    async fn marketplaces(&self, ctx: &Context<'_>) -> FieldResult<Vec<MarketplaceSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .marketplaces()
            .fetch_marketplaces()
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(guard = "UserGuard")]
    async fn activities(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctActivitySchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryActivitySchema,
        #[graphql(default, name = "order_by")] order: OrderActivitySchema,
    ) -> FieldResult<Vec<ActivitySchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .activities()
            .fetch_activities(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "activities_aggregate", guard = "UserGuard")]
    async fn activities_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctActivitySchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryActivitySchema,
        #[graphql(default, name = "order_by")] order: OrderActivitySchema,
    ) -> FieldResult<AggregateSchema<AggregateActivitySchema, ActivitySchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let selection = get_aggregate_selection(ctx);

        let aggregate = db
            .activities()
            .fetch_aggregate_activities(&selection.aggregate, &query, distinct.as_ref())
            .await?;

        if selection.nodes.is_empty() {
            return Ok(AggregateSchema {
                aggregate,
                nodes: Vec::new(),
            });
        }

        let nodes = db
            .activities()
            .fetch_activities(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema { aggregate, nodes })
    }

    #[graphql(guard = "UserGuard")]
    async fn attributes(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctAttributeSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryAttributeSchema,
        #[graphql(default, name = "order_by")] order: OrderAttributeSchema,
    ) -> FieldResult<Vec<AttributeSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .attributes()
            .fetch_attributes(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "attributes_aggregate", guard = "UserGuard")]
    async fn attributes_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctAttributeSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryAttributeSchema,
        #[graphql(default, name = "order_by")] order: OrderAttributeSchema,
    ) -> FieldResult<AggregateSchema<AggregateAttributeSchema, AttributeSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let selection = get_aggregate_selection(ctx);

        let aggregate = db
            .attributes()
            .fetch_aggregate_attributes(&selection.aggregate, &query, distinct.as_ref())
            .await?;

        if selection.nodes.is_empty() {
            return Ok(AggregateSchema {
                aggregate,
                nodes: Vec::new(),
            });
        }

        let nodes = db
            .attributes()
            .fetch_attributes(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema { aggregate, nodes })
    }

    #[graphql(guard = "UserGuard")]
    async fn bids(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctBidSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryBidSchema,
        #[graphql(default, name = "order_by")] order: OrderBidSchema,
    ) -> FieldResult<Vec<BidSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .bids()
            .fetch_bids(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "bids_aggregate", guard = "UserGuard")]
    async fn bids_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctBidSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryBidSchema,
        #[graphql(default, name = "order_by")] order: OrderBidSchema,
    ) -> FieldResult<AggregateSchema<AggregateBidSchema, BidSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let selection = get_aggregate_selection(ctx);

        let aggregate = db
            .bids()
            .fetch_aggregate_bids(&selection.aggregate, &query, distinct.as_ref())
            .await?;

        if selection.nodes.is_empty() {
            return Ok(AggregateSchema {
                aggregate,
                nodes: Vec::new(),
            });
        }

        let nodes = db
            .bids()
            .fetch_bids(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema { aggregate, nodes })
    }

    #[graphql(guard = "UserGuard")]
    async fn collections(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctCollectionSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryCollectionSchema,
        #[graphql(default, name = "order_by")] order: OrderCollectionSchema,
    ) -> FieldResult<Vec<CollectionSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_collections(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collections_aggregate", guard = "UserGuard")]
    async fn collections_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctCollectionSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryCollectionSchema,
        #[graphql(default, name = "order_by")] order: OrderCollectionSchema,
    ) -> FieldResult<AggregateSchema<AggregateCollectionSchema, CollectionSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let selection = get_aggregate_selection(ctx);

        let aggregate = db
            .collections()
            .fetch_aggregate_collections(&selection.aggregate, &query, distinct.as_ref())
            .await?;

        if selection.nodes.is_empty() {
            return Ok(AggregateSchema {
                aggregate,
                nodes: Vec::new(),
            });
        }

        let nodes = db
            .collections()
            .fetch_collections(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema { aggregate, nodes })
    }

    #[graphql(guard = "UserGuard")]
    async fn listings(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctListingSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryListingSchema,
        #[graphql(default, name = "order_by")] order: OrderListingSchema,
    ) -> FieldResult<Vec<ListingSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .listings()
            .fetch_listings(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "listings_aggregate", guard = "UserGuard")]
    async fn listings_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctListingSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryListingSchema,
        #[graphql(default, name = "order_by")] order: OrderListingSchema,
    ) -> FieldResult<AggregateSchema<AggregateListingSchema, ListingSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let selection = get_aggregate_selection(ctx);

        let aggregate = db
            .listings()
            .fetch_aggregate_listings(&selection.aggregate, &query, distinct.as_ref())
            .await?;

        if selection.nodes.is_empty() {
            return Ok(AggregateSchema {
                aggregate,
                nodes: Vec::new(),
            });
        }

        let nodes = db
            .listings()
            .fetch_listings(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema { aggregate, nodes })
    }

    #[graphql(guard = "UserGuard")]
    async fn nfts(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctNftSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryNftSchema,
        #[graphql(default, name = "order_by")] order: OrderNftSchema,
    ) -> FieldResult<Vec<NftSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .nfts()
            .fetch_nfts(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "nfts_aggregate", guard = "UserGuard")]
    async fn nfts_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctNftSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryNftSchema,
        #[graphql(default, name = "order_by")] order: OrderNftSchema,
    ) -> FieldResult<AggregateSchema<AggregateNftSchema, NftSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let selection = get_aggregate_selection(ctx);

        let aggregate = db
            .nfts()
            .fetch_aggregate_nfts(&selection.aggregate, &query, distinct.as_ref())
            .await?;

        if selection.nodes.is_empty() {
            return Ok(AggregateSchema {
                aggregate,
                nodes: Vec::new(),
            });
        }

        let nodes = db
            .nfts()
            .fetch_nfts(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema { aggregate, nodes })
    }

    #[graphql(name = "collection_trendings", guard = "UserGuard")]
    async fn collection_trendings(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(
            desc = "The available unit is `d (days)`, `h (hours)`, `m (minutes)`, and `s (seconds)`"
        )]
        period: Option<Wrapper<PgInterval>>,
        #[graphql(default_with = "OrderTrendingType::default()", name = "trending_by")]
        order: OrderTrendingType,
    ) -> FieldResult<Vec<CollectionTrendingSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_trendings(limit, offset, order, period.map(|w| w.0))
            .await
            .map_err(|e| FieldError::from(e))
    }

    // ==================== WALLET ====================
    #[graphql(name = "wallet_stats", guard = "UserGuard")]
    async fn wallet_stats(&self, ctx: &Context<'_>, address: String) -> FieldResult<StatsSchema> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .wallets()
            .fetch_stats(&address)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "wallet_nft_holding_period", guard = "UserGuard")]
    async fn wallet_nft_holding_period(
        &self,
        ctx: &Context<'_>,
        address: String,
    ) -> FieldResult<Vec<NftHoldingPeriodSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .wallets()
            .fetch_nft_holding_periods(&address)
            .await
            .map_err(|e| FieldError::from(e))
    }
    // ================================================

    // ============= COLLECTION ANALYTICS =============
    #[graphql(name = "collection_holders", guard = "UserGuard")]
    async fn collection_holders(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default_with = "OrderHolderType::default()", name = "trending_by")]
        order: OrderHolderType,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<CollectionHolderSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_holders(collection_id, order, limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_stats", guard = "UserGuard")]
    async fn collection_stats(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<CollectionStatSchema> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_stats(collection_id)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_trending_nfts", guard = "UserGuard")]
    async fn collection_trending_nfts(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<TrendingNftSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_trending_nfts(collection_id, limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_nft_changes", guard = "UserGuard")]
    async fn collection_nft_changes(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(
            desc = "The available unit is `d (days)`, `h (hours)`, `m (minutes)`, and `s (seconds)`"
        )]
        interval: Option<Wrapper<PgInterval>>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<NftChangeSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_nft_changes(collection_id, limit, offset, interval.map(|w| w.0))
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_profit_leaderboards", guard = "UserGuard")]
    async fn collection_profit_leaderboards(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<ProfitLeaderboardSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_profit_leaderboards(collection_id, limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_top_wallets", guard = "UserGuard")]
    async fn collection_top_wallets(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(
            desc = "The available unit is `d (days)`, `h (hours)`, `m (minutes)`, and `s (seconds)`"
        )]
        interval: Option<Wrapper<PgInterval>>,
        #[graphql(name = "type")] type_: TopWalletType,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<TopWalletSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_top_wallets(collection_id, type_, limit, interval.map(|w| w.0))
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_floor_charts", guard = "UserGuard")]
    async fn collection_floor_charts(
        &self,
        ctx: &Context<'_>,
        #[graphql(
            name = "start_time",
            desc = "The value can be a date string or unix in milliseconds"
        )]
        start_time: Wrapper<DateTime<Utc>>,
        #[graphql(
            name = "end_time",
            desc = "The value can be a date string or unix in milliseconds"
        )]
        end_time: Wrapper<DateTime<Utc>>,
        #[graphql(
            desc = "The available unit is `d (days)`, `h (hours)`, `m (minutes)`, and `s (seconds)`"
        )]
        interval: Wrapper<PgInterval>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<DataPointSchema>> {
        if !validate_data_set(&start_time.0, &end_time.0, &interval.0) {
            return Err(FieldError::new(
                "The requested dataset is too large to process. Please reduce the time range or interval.",
            ));
        }

        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_floor_charts(collection_id, start_time.0, end_time.0, interval.0)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_volume_charts", guard = "UserGuard")]
    async fn collection_volume_charts(
        &self,
        ctx: &Context<'_>,
        #[graphql(
            name = "start_time",
            desc = "The value can be a date string or unix in milliseconds"
        )]
        start_time: Wrapper<DateTime<Utc>>,
        #[graphql(
            name = "end_time",
            desc = "The value can be a date string or unix in milliseconds"
        )]
        end_time: Wrapper<DateTime<Utc>>,
        #[graphql(
            desc = "The available unit is `d (days)`, `h (hours)`, `m (minutes)`, and `s (seconds)`"
        )]
        interval: Wrapper<PgInterval>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
        #[graphql(name = "type")] coin_type: CoinType,
    ) -> FieldResult<Vec<DataPointSchema>> {
        if !validate_data_set(&start_time.0, &end_time.0, &interval.0) {
            return Err(FieldError::new(
                "The requested dataset is too large to process. Please reduce the time range or interval.",
            ));
        }

        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_volume_charts(
                collection_id,
                start_time.0,
                end_time.0,
                interval.0,
                coin_type,
            )
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_nft_holders", guard = "UserGuard")]
    async fn collection_nft_holders(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<NftHolderSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_nft_holders(collection_id, limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_attributes", guard = "UserGuard")]
    async fn collection_attributes(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<CollectionAttributeSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_attributes(collection_id)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_nft_amount_distribution", guard = "UserGuard")]
    async fn collection_nft_amount_distribution(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<NftAmountDistributionSchema> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_nft_amount_distribution(collection_id)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_nft_period_distribution", guard = "UserGuard")]
    async fn collection_nft_period_distribution(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<NftPeriodDistributionSchema> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .collections()
            .fetch_nft_period_distribution(collection_id)
            .await
            .map_err(|e| FieldError::from(e))
    }

    // ================================================

    // ================== Activities ==================
    #[graphql(name = "activity_profit_losses", guard = "UserGuard")]
    async fn activity_profit_losses(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(name = "wallet_address")] wallet_address: String,
    ) -> FieldResult<Vec<ProfitLossSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .activities()
            .fetch_profit_and_loss(&wallet_address, limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "activity_contribution_charts", guard = "UserGuard")]
    async fn activity_contribution_charts(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "wallet_address")] wallet_address: String,
    ) -> FieldResult<Vec<DataPointSchema>> {
        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .activities()
            .fetch_contribution_chart(&wallet_address)
            .await
            .map_err(|e| FieldError::from(e))
    }
}
