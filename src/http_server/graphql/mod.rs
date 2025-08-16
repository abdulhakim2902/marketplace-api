pub mod guard;

use std::sync::Arc;

use crate::database::attributes::IAttributes;
use crate::models::schema::attribute::{AttributeSchema, FilterAttributeSchema};
use crate::models::schema::collection::nft_holder::FilterNftHolderSchema;
use crate::models::schema::collection::stat::CollectionStatSchema;
use crate::models::schema::data_point::FilterFloorChartSchema;
use crate::{
    database::{
        Database, IDatabase, activities::IActivities, bids::IBids, collections::ICollections,
        listings::IListings, marketplaces::IMarketplaces, nfts::INfts, wallets::IWallets,
    },
    models::schema::{
        activity::{
            ActivitySchema, FilterActivitySchema,
            profit_loss::{FilterProfitLossSchema, ProfitLossSchema},
        },
        collection::{
            CollectionSchema, FilterCollectionSchema,
            attribute::CollectionAttributeSchema,
            nft_change::{FilterNftChangeSchema, NftChangeSchema},
            nft_distribution::{NftAmountDistributionSchema, NftPeriodDistributionSchema},
            nft_holder::NftHolderSchema,
            profit_leaderboard::{FilterLeaderboardSchema, ProfitLeaderboardSchema},
            top_wallet::{FilterTopWalletSchema, TopWalletSchema},
            trending::{FilterTrendingSchema, TrendingSchema},
        },
        data_point::DataPointSchema,
        listing::{FilterListingSchema, ListingSchema},
        marketplace::MarketplaceSchema,
        nft::{FilterNftSchema, NftSchema},
        offer::{FilterOfferSchema, OfferSchema},
        wallet::{nft_holding_period::NftHoldingPeriod, stats::StatsSchema},
    },
};
use async_graphql::{Context, Object, http::GraphiQLSource};
use axum::response::{Html, IntoResponse};

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
        filter: Option<FilterAttributeSchema>,
    ) -> Vec<AttributeSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.attributes()
            .fetch_attributes(filter.unwrap_or_default())
            .await
            .expect("Failed to fetch attributes")
    }

    async fn activities(
        &self,
        ctx: &Context<'_>,
        filter: Option<FilterActivitySchema>,
    ) -> Vec<ActivitySchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.activities()
            .fetch_activities(filter.unwrap_or_default())
            .await
            .expect("Failed to fetch activities")
    }

    async fn collections(
        &self,
        ctx: &Context<'_>,
        filter: Option<FilterCollectionSchema>,
    ) -> Vec<CollectionSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_collections(filter.unwrap_or_default())
            .await
            .expect("Failed to fetch collections")
    }

    async fn nfts(&self, ctx: &Context<'_>, filter: Option<FilterNftSchema>) -> Vec<NftSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.nfts()
            .fetch_nfts(filter.unwrap_or_default())
            .await
            .expect("Failed to fetch nfts")
    }

    async fn listings(
        &self,
        ctx: &Context<'_>,
        filter: Option<FilterListingSchema>,
    ) -> Vec<ListingSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.listings()
            .fetch_listings(filter)
            .await
            .expect("Failed to fetch nfts")
    }

    async fn bids(&self, ctx: &Context<'_>, filter: Option<FilterOfferSchema>) -> Vec<OfferSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.bids()
            .fetch_bids(filter)
            .await
            .expect("Failed to fetch bids")
    }

    // ==================== WALLET ====================
    async fn wallet_stats(&self, ctx: &Context<'_>, address: String) -> Option<StatsSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.wallets().fetch_stats(&address).await.ok()
    }

    async fn wallet_nft_holding_period(
        &self,
        ctx: &Context<'_>,
        address: String,
    ) -> Vec<NftHoldingPeriod> {
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
    async fn collection_stat(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
    ) -> CollectionStatSchema {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_stat(&collection_id)
            .await
            .expect("Failed to fetch collection stat")
    }

    async fn collection_trending(
        &self,
        ctx: &Context<'_>,
        filter: FilterTrendingSchema,
    ) -> Vec<TrendingSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_trending(filter)
            .await
            .expect("Failed to fetch collection trending")
    }

    async fn collection_nft_changes(
        &self,
        ctx: &Context<'_>,
        filter: FilterNftChangeSchema,
    ) -> Vec<NftChangeSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_nft_changes(filter)
            .await
            .expect("Failed to fetch collection nft period distribution")
    }

    async fn collection_profit_leaderboards(
        &self,
        ctx: &Context<'_>,
        filter: FilterLeaderboardSchema,
    ) -> Vec<ProfitLeaderboardSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_profit_leaderboards(filter)
            .await
            .expect("Failed to fetch collection nft period distribution")
    }

    async fn collection_top_wallets(
        &self,
        ctx: &Context<'_>,
        filter: FilterTopWalletSchema,
    ) -> Vec<TopWalletSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_top_wallets(filter)
            .await
            .expect("Failed to fetch collection top buyers")
    }

    async fn collection_floor_charts(
        &self,
        ctx: &Context<'_>,
        filter: FilterFloorChartSchema,
    ) -> Vec<DataPointSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_floor_charts(filter)
            .await
            .expect("Failed to fetch floor chart")
    }

    async fn collection_nft_holders(
        &self,
        ctx: &Context<'_>,
        filter: FilterNftHolderSchema,
    ) -> Vec<NftHolderSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_nft_holders(filter)
            .await
            .expect("Failed to fetch nft holders")
    }

    async fn collection_attributes(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
    ) -> Vec<CollectionAttributeSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_attributes(&collection_id)
            .await
            .expect("Failed to fetch collection attributes")
    }

    async fn collection_nft_amount_distribution(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
    ) -> Option<NftAmountDistributionSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_nft_amount_distribution(&collection_id)
            .await
            .ok()
    }

    async fn collection_nft_period_distribution(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Option<NftPeriodDistributionSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.collections()
            .fetch_nft_period_distribution(&id)
            .await
            .ok()
    }

    // ================================================

    // ================== Activities ==================
    async fn profit_loss_activities(
        &self,
        ctx: &Context<'_>,
        filter: Option<FilterProfitLossSchema>,
    ) -> Vec<ProfitLossSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.activities()
            .fetch_profit_and_loss(filter)
            .await
            .expect("Failed to fetch wallet profit loss")
    }

    async fn contribution_chart_activities(
        &self,
        ctx: &Context<'_>,
        wallet_address: String,
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
