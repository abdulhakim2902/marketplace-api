pub mod guard;

use std::sync::Arc;

use crate::{
    database::{
        Database, IDatabase, activities::IActivities, attributes::IAttributes, bids::IBids,
        collections::ICollections, listings::IListings, marketplaces::IMarketplaces, nfts::INfts,
        wallets::IWallets,
    },
    http_server::graphql::guard::UserGuard,
    models::schema::{
        AggregateSchema, CoinType,
        activity::{
            ActivitySchema, DistinctActivitySchema, OrderActivitySchema, QueryActivitySchema,
            profit_loss::ProfitLossSchema,
        },
        attribute::{
            AttributeSchema, DistinctAttributeSchema, OrderAttributeSchema, QueryAttributeSchema,
        },
        bid::{BidSchema, DistinctBidSchema, OrderBidSchema, QueryBidSchema},
        collection::{
            CollectionSchema, DistinctCollectionSchema, OrderCollectionSchema,
            QueryCollectionSchema,
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
        listing::{DistinctListingSchema, ListingSchema, OrderListingSchema, QueryListingSchema},
        marketplace::MarketplaceSchema,
        nft::{DistinctNftSchema, NftSchema, OrderNftSchema, QueryNftSchema},
        wallet::{nft_holding_period::NftHoldingPeriodSchema, stats::StatsSchema},
    },
    utils::string_utils,
};
use async_graphql::{
    Context, FieldError, FieldResult, InputValueError, InputValueResult, Object, Scalar,
    ScalarType, Value, http::GraphiQLSource,
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
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        db.marketplaces()
            .fetch_marketplaces()
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(guard = "UserGuard")]
    async fn attributes(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctAttributeSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryAttributeSchema>,
        #[graphql(name = "order_by")] order: Option<OrderAttributeSchema>,
    ) -> FieldResult<Vec<AttributeSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        db.attributes()
            .fetch_attributes(&distinct, limit, offset, &query, &order)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "attributes_aggregate", guard = "UserGuard")]
    async fn attributes_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctAttributeSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryAttributeSchema>,
        #[graphql(name = "order_by")] order: Option<OrderAttributeSchema>,
    ) -> FieldResult<AggregateSchema<AttributeSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        let total = db
            .attributes()
            .fetch_total_attributes(&distinct, &query)
            .await
            .map_err(|e| FieldError::from(e))?;

        let nodes = db
            .attributes()
            .fetch_attributes(&distinct, limit, offset, &query, &order)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema::new(total, nodes))
    }

    #[graphql(guard = "UserGuard")]
    async fn activities(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctActivitySchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryActivitySchema>,
        #[graphql(name = "order_by")] order: Option<OrderActivitySchema>,
    ) -> FieldResult<Vec<ActivitySchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        db.activities()
            .fetch_activities(&distinct, limit, offset, &query, &order)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "activities_aggregate", guard = "UserGuard")]
    async fn activities_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctActivitySchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryActivitySchema>,
        #[graphql(name = "order_by")] order: Option<OrderActivitySchema>,
    ) -> FieldResult<AggregateSchema<ActivitySchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        let total = db
            .activities()
            .fetch_total_activities(&distinct, &query)
            .await
            .map_err(|e| FieldError::from(e))?;

        let nodes = db
            .activities()
            .fetch_activities(&distinct, limit, offset, &query, &order)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema::new(total, nodes))
    }

    #[graphql(guard = "UserGuard")]
    async fn collections(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctCollectionSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryCollectionSchema>,
        #[graphql(name = "order_by")] order: Option<OrderCollectionSchema>,
    ) -> FieldResult<Vec<CollectionSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        db.collections()
            .fetch_collections(&distinct, limit, offset, &query, &order)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collections_aggregate", guard = "UserGuard")]
    async fn collections_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctCollectionSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryCollectionSchema>,
        #[graphql(name = "order_by")] order: Option<OrderCollectionSchema>,
    ) -> FieldResult<AggregateSchema<CollectionSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        let total = db
            .collections()
            .fetch_total_collections(&distinct, &query)
            .await
            .map_err(|e| FieldError::from(e))?;

        let nodes = db
            .collections()
            .fetch_collections(&distinct, limit, offset, &query, &order)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema::new(total, nodes))
    }

    #[graphql(guard = "UserGuard")]
    async fn nfts(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctNftSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryNftSchema>,
        #[graphql(name = "order_by")] order: Option<OrderNftSchema>,
    ) -> FieldResult<Vec<NftSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        db.nfts()
            .fetch_nfts(&distinct, limit, offset, &query, &order)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "nfts_aggregate", guard = "UserGuard")]
    async fn nfts_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctNftSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryNftSchema>,
        #[graphql(name = "order_by")] order: Option<OrderNftSchema>,
    ) -> FieldResult<AggregateSchema<NftSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        let total = db
            .nfts()
            .fetch_total_nfts(&distinct, &query)
            .await
            .map_err(|e| FieldError::from(e))?;

        let nodes = db
            .nfts()
            .fetch_nfts(&distinct, limit, offset, &query, &order)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema::new(total, nodes))
    }

    #[graphql(guard = "UserGuard")]
    async fn listings(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctListingSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryListingSchema>,
        #[graphql(name = "order_by")] order: Option<OrderListingSchema>,
    ) -> FieldResult<Vec<ListingSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        db.listings()
            .fetch_listings(&distinct, limit, offset, &query, &order)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "listings_aggregate", guard = "UserGuard")]
    async fn listings_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctListingSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryListingSchema>,
        #[graphql(name = "order_by")] order: Option<OrderListingSchema>,
    ) -> FieldResult<AggregateSchema<ListingSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        let total = db
            .listings()
            .fetch_total_listings(&distinct, &query)
            .await
            .map_err(|e| FieldError::from(e))?;

        let nodes = db
            .listings()
            .fetch_listings(&distinct, limit, offset, &query, &order)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema::new(total, nodes))
    }

    #[graphql(guard = "UserGuard")]
    async fn bids(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctBidSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryBidSchema>,
        #[graphql(name = "order_by")] order: Option<OrderBidSchema>,
    ) -> FieldResult<Vec<BidSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        db.bids()
            .fetch_bids(&distinct, limit, offset, &query, &order)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "bids_aggregate", guard = "UserGuard")]
    async fn bids_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctBidSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryBidSchema>,
        #[graphql(name = "order_by")] order: Option<OrderBidSchema>,
    ) -> FieldResult<AggregateSchema<BidSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        let total = db
            .bids()
            .fetch_total_bids(&distinct, &query)
            .await
            .map_err(|e| FieldError::from(e))?;

        let nodes = db
            .bids()
            .fetch_bids(&distinct, limit, offset, &query, &order)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema::new(total, nodes))
    }

    #[graphql(name = "collection_trendings", guard = "UserGuard")]
    async fn collection_trendings(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(
            desc = "The available unit is `d (days)`, `h (hours)`, `m (minutes)`, and `s (seconds)`"
        )]
        period: Option<Wrapper<PgInterval>>,
        #[graphql(name = "trending_by")] order: Option<OrderTrendingType>,
    ) -> FieldResult<Vec<CollectionTrendingSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        db.collections()
            .fetch_trendings(limit, offset, order, period.map(|w| w.0))
            .await
            .map_err(|e| FieldError::from(e))
    }

    // ==================== WALLET ====================
    #[graphql(name = "wallet_stats", guard = "UserGuard")]
    async fn wallet_stats(&self, ctx: &Context<'_>, address: String) -> FieldResult<StatsSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        db.wallets()
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
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        db.wallets()
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
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "trending_by")] order: Option<OrderHolderType>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<CollectionHolderSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let order = order.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.collections()
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
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        db.collections()
            .fetch_stats(collection_id)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_trending_nfts", guard = "UserGuard")]
    async fn collection_trending_nfts(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<TrendingNftSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.collections()
            .fetch_trending_nfts(collection_id, limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_nft_changes", guard = "UserGuard")]
    async fn collection_nft_changes(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(
            desc = "The available unit is `d (days)`, `h (hours)`, `m (minutes)`, and `s (seconds)`"
        )]
        interval: Option<Wrapper<PgInterval>>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<NftChangeSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.collections()
            .fetch_nft_changes(collection_id, limit, offset, interval.map(|w| w.0))
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_profit_leaderboards", guard = "UserGuard")]
    async fn collection_profit_leaderboards(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<ProfitLeaderboardSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.collections()
            .fetch_profit_leaderboards(collection_id, limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "collection_top_wallets", guard = "UserGuard")]
    async fn collection_top_wallets(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        #[graphql(
            desc = "The available unit is `d (days)`, `h (hours)`, `m (minutes)`, and `s (seconds)`"
        )]
        interval: Option<Wrapper<PgInterval>>,
        #[graphql(name = "type")] type_: TopWalletType,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<TopWalletSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let limit = limit.unwrap_or(10);

        db.collections()
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
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        if !validate_data_set(&start_time.0, &end_time.0, &interval.0) {
            return Err(FieldError::new(
                "The requested dataset is too large to process. Please reduce the time range or interval.",
            ));
        }

        db.collections()
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
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        if !validate_data_set(&start_time.0, &end_time.0, &interval.0) {
            return Err(FieldError::new(
                "The requested dataset is too large to process. Please reduce the time range or interval.",
            ));
        }

        db.collections()
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
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> FieldResult<Vec<NftHolderSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.collections()
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
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        db.collections()
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
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        db.collections()
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
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        db.collections()
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
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "wallet_address")] wallet_address: String,
    ) -> FieldResult<Vec<ProfitLossSchema>> {
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.activities()
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
        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        db.activities()
            .fetch_contribution_chart(&wallet_address)
            .await
            .map_err(|e| FieldError::from(e))
    }
}
