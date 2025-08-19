pub mod guard;

use std::sync::Arc;

use crate::{
    database::{
        Database, IDatabase, activities::IActivities, attributes::IAttributes, bids::IBids,
        collections::ICollections, listings::IListings, marketplaces::IMarketplaces, nfts::INfts,
        wallets::IWallets,
    },
    models::schema::{
        activity::{
            ActivitySchema, OrderActivitySchema, QueryActivitySchema, profit_loss::ProfitLossSchema,
        },
        attribute::{AttributeSchema, OrderAttributeSchema, QueryAttributeSchema},
        bid::{BidSchema, OrderBidSchema, QueryBidSchema},
        collection::{
            CollectionSchema, OrderCollectionSchema, QueryCollectionSchema,
            attribute::CollectionAttributeSchema,
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
        listing::{ListingSchema, OrderListingSchema, QueryListingSchema},
        marketplace::MarketplaceSchema,
        nft::{NftSchema, OrderNftSchema, QueryNftSchema},
        wallet::{nft_holding_period::NftHoldingPeriodSchema, stats::StatsSchema},
    },
};
use async_graphql::{Context, Object, http::GraphiQLSource};
use axum::response::{Html, IntoResponse};
use uuid::Uuid;

pub async fn graphql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/gql").finish())
}

pub struct Query;

#[Object]
impl Query {
    async fn marketplaces(&self, ctx: &Context<'_>) -> Vec<MarketplaceSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.marketplaces()
            .fetch_marketplaces()
            .await
            .expect("Failed to fetch marketplaces")
    }

    async fn attributes(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryAttributeSchema>,
        #[graphql(name = "order_by")] order: Option<OrderAttributeSchema>,
    ) -> Vec<AttributeSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        db.attributes()
            .fetch_attributes(limit, offset, query, order)
            .await
            .expect("Failed to fetch attributes")
    }

    async fn activities(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryActivitySchema>,
        #[graphql(name = "order_by")] order: Option<OrderActivitySchema>,
    ) -> Vec<ActivitySchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        db.activities()
            .fetch_activities(limit, offset, query, order)
            .await
            .expect("Failed to fetch activities")
    }

    async fn collections(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryCollectionSchema>,
        #[graphql(name = "order_by")] order: Option<OrderCollectionSchema>,
    ) -> Vec<CollectionSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        db.collections()
            .fetch_collections(limit, offset, query, order)
            .await
            .expect("Failed to fetch collections")
    }

    async fn nfts(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryNftSchema>,
        #[graphql(name = "order_by")] order: Option<OrderNftSchema>,
    ) -> Vec<NftSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        db.nfts()
            .fetch_nfts(limit, offset, query, order)
            .await
            .expect("Failed to fetch nfts")
    }

    async fn listings(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryListingSchema>,
        #[graphql(name = "order_by")] order: Option<OrderListingSchema>,
    ) -> Vec<ListingSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        db.listings()
            .fetch_listings(limit, offset, query, order)
            .await
            .expect("Failed to fetch nfts")
    }

    async fn bids(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryBidSchema>,
        #[graphql(name = "order_by")] order: Option<OrderBidSchema>,
    ) -> Vec<BidSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let query = query.unwrap_or_default();
        let order = order.unwrap_or_default();

        db.bids()
            .fetch_bids(limit, offset, query, order)
            .await
            .expect("Failed to fetch bids")
    }

    #[graphql(name = "collection_trendings")]
    async fn collection_trendings(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        interval: Option<String>,
        #[graphql(name = "order_by")] order: Option<OrderTrendingType>,
    ) -> Vec<CollectionTrendingSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let interval = interval.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        db.collections()
            .fetch_trendings(&interval, limit, offset, order)
            .await
            .expect("Failed to fetch collection trendings")
    }

    // ==================== WALLET ====================
    #[graphql(name = "wallet_stats")]
    async fn wallet_stats(&self, ctx: &Context<'_>, address: String) -> Option<StatsSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.wallets().fetch_stats(&address).await.ok()
    }

    #[graphql(name = "wallet_nft_holding_period")]
    async fn wallet_nft_holding_period(
        &self,
        ctx: &Context<'_>,
        address: String,
    ) -> Vec<NftHoldingPeriodSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.wallets()
            .fetch_nft_holding_periods(&address)
            .await
            .expect("Failed to fetch wallet holding")
    }
    // ================================================

    // ============= COLLECTION ANALYTICS =============
    #[graphql(name = "collection_stats")]
    async fn collection_stats(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> CollectionStatSchema {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_stats(collection_id)
            .await
            .expect("Failed to fetch collection stats")
    }

    #[graphql(name = "collection_trending_nfts")]
    async fn collection_trending_nfts(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> Vec<TrendingNftSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.collections()
            .fetch_trending_nfts(collection_id, limit, offset)
            .await
            .expect("Failed to fetch collection trending")
    }

    #[graphql(name = "collection_nft_changes")]
    async fn collection_nft_changes(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        interval: Option<String>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> Vec<NftChangeSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.collections()
            .fetch_nft_changes(collection_id, limit, offset, interval)
            .await
            .expect("Failed to fetch collection nft period distribution")
    }

    #[graphql(name = "collection_profit_leaderboards")]
    async fn collection_profit_leaderboards(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> Vec<ProfitLeaderboardSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.collections()
            .fetch_profit_leaderboards(collection_id, limit, offset)
            .await
            .expect("Failed to fetch collection nft period distribution")
    }

    #[graphql(name = "collection_top_wallets")]
    async fn collection_top_wallets(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        interval: Option<String>,
        #[graphql(name = "type")] type_: TopWalletType,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> Vec<TopWalletSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);

        db.collections()
            .fetch_top_wallets(collection_id, type_, limit, interval)
            .await
            .expect("Failed to fetch collection top buyers")
    }

    #[graphql(name = "collection_floor_charts")]
    async fn collection_floor_charts(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "start_time")] start_time: i64,
        #[graphql(name = "end_time")] end_time: i64,
        interval: String,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> Vec<DataPointSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_floor_charts(collection_id, start_time, end_time, &interval)
            .await
            .expect("Failed to fetch floor chart")
    }

    #[graphql(name = "collection_nft_holders")]
    async fn collection_nft_holders(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> Vec<NftHolderSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.collections()
            .fetch_nft_holders(collection_id, limit, offset)
            .await
            .expect("Failed to fetch nft holders")
    }

    #[graphql(name = "collection_attributes")]
    async fn collection_attributes(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> Vec<CollectionAttributeSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_attributes(collection_id)
            .await
            .expect("Failed to fetch collection attributes")
    }

    #[graphql(name = "collection_nft_amount_distribution")]
    async fn collection_nft_amount_distribution(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> Option<NftAmountDistributionSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_nft_amount_distribution(collection_id)
            .await
            .ok()
    }

    #[graphql(name = "collection_nft_period_distribution")]
    async fn collection_nft_period_distribution(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "collection_id")] collection_id: Uuid,
    ) -> Option<NftPeriodDistributionSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_nft_period_distribution(collection_id)
            .await
            .ok()
    }

    // ================================================

    // ================== Activities ==================
    #[graphql(name = "activity_profit_losses")]
    async fn activity_profit_losses(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "wallet_address")] wallet_address: String,
    ) -> Vec<ProfitLossSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.activities()
            .fetch_profit_and_loss(&wallet_address, limit, offset)
            .await
            .expect("Failed to fetch wallet profit loss")
    }

    #[graphql(name = "activity_contribution_charts")]
    async fn activity_contribution_charts(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "wallet_address")] wallet_address: String,
    ) -> Vec<DataPointSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.activities()
            .fetch_contribution_chart(&wallet_address)
            .await
            .expect("Failed to fetch contribution chart")
    }
}
