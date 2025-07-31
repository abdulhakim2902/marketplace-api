use std::sync::Arc;

use crate::{
    database::{
        Database, IDatabase, activities::IActivities, attributes::IAttributes, bids::IBids,
        collections::ICollections, listings::IListings, marketplaces::IMarketplaces, nfts::INfts,
        wallets::IWallets,
    },
    models::schema::{
        activity::{ActivitySchema, FilterActivitySchema},
        attribute::CollectionAttributeSchema,
        bid::{BidSchema, FilterBidSchema},
        collection::{CollectionSchema, FilterCollectionSchema},
        collection_trending::{CollectionTrendingSchema, FilterCollectionTrendingSchema},
        data_point::DataPointSchema,
        listing::{FilterListingSchema, ListingSchema},
        marketplace::MarketplaceSchema,
        nft::{FilterNftSchema, NftSchema},
        nft_change::{FilterNftChangeSchema, NftChangeSchema},
        nft_distribution::{NftAmountDistributionSchema, NftPeriodDistributionSchema},
        nft_holder::NftHolderSchema,
        profit_leaderboard::{FilterLeaderboardSchema, ProfitLeaderboardSchema},
        profit_loss_activity::{FilterProfitLossActivitySchema, ProfitLossActivitySchema},
        top_buyer::TopBuyerSchema,
        top_seller::TopSellerSchema,
        wallet_stat::WalletStatSchema,
    },
    utils::string_utils,
};
use async_graphql::{Context, Object, http::GraphiQLSource};
use axum::response::{Html, IntoResponse};
use chrono::DateTime;

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

    async fn activities(
        &self,
        ctx: &Context<'_>,
        filter: Option<FilterActivitySchema>,
    ) -> Vec<ActivitySchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let filter = filter.unwrap_or_default();
        let query = filter.where_.unwrap_or_default();

        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        db.activities()
            .fetch_activities(&query, limit, offset)
            .await
            .expect("Failed to fetch activites")
    }

    async fn collections(
        &self,
        ctx: &Context<'_>,
        filter: Option<FilterCollectionSchema>,
    ) -> Vec<CollectionSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let filter = filter.unwrap_or_default();
        let query = filter.where_.unwrap_or_default();

        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        db.collections()
            .fetch_collections(&query, limit, offset)
            .await
            .expect("Failed to fetch collections")
    }

    async fn nfts(&self, ctx: &Context<'_>, filter: Option<FilterNftSchema>) -> Vec<NftSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let filter = filter.unwrap_or_default();
        let query = filter.where_.unwrap_or_default();

        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        db.nfts()
            .fetch_nfts(&query, limit, offset)
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

        let filter = filter.unwrap_or_default();
        let query = filter.where_.unwrap_or_default();

        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        db.listings()
            .fetch_listings(&query, limit, offset)
            .await
            .expect("Failed to fetch nfts")
    }

    async fn offers(&self, ctx: &Context<'_>, filter: Option<FilterBidSchema>) -> Vec<BidSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let filter = filter.unwrap_or_default();
        let query = filter.where_.unwrap_or_default();

        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        db.bids()
            .fetch_bids(&query, limit, offset)
            .await
            .expect("Failed to fetch bids")
    }

    async fn profit_loss_activities(
        &self,
        ctx: &Context<'_>,
        filter: Option<FilterProfitLossActivitySchema>,
    ) -> Vec<ProfitLossActivitySchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let filter = filter.unwrap_or_default();
        let query = filter.where_.unwrap_or_default();

        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        db.activities()
            .fetch_profit_and_loss(&query, limit, offset)
            .await
            .expect("Failed to fetch wallet profit loss")
    }

    async fn wallet_stat(&self, ctx: &Context<'_>, address: String) -> Option<WalletStatSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.wallets().fetch_stat(&address).await.ok()
    }

    async fn collection_trending(
        &self,
        ctx: &Context<'_>,
        filter: FilterCollectionTrendingSchema,
    ) -> Vec<CollectionTrendingSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let query = filter.where_;

        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        db.collections()
            .fetch_collection_trending(&query, limit, offset)
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

        let query = filter.where_;

        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        db.activities()
            .fetch_nft_changes(&query, limit, offset)
            .await
            .expect("Failed to fetch collection nft period distribution")
    }

    async fn collection_profit_leaderboard(
        &self,
        ctx: &Context<'_>,
        filter: FilterLeaderboardSchema,
    ) -> Vec<ProfitLeaderboardSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let query = filter.where_;

        let limit = filter.limit.unwrap_or(10);
        let offset = filter.offset.unwrap_or(0);

        db.activities()
            .fetch_profit_leaderboard(&query, limit, offset)
            .await
            .expect("Failed to fetch collection nft period distribution")
    }

    async fn collection_attributes(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
    ) -> Vec<CollectionAttributeSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.attributes()
            .fetch_collection_attributes(&collection_id)
            .await
            .expect("Failed to fetch collection attributes")
    }

    async fn collection_top_buyer(
        &self,
        ctx: &Context<'_>,
        id: String,
        interval: Option<String>,
    ) -> Vec<TopBuyerSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let i = string_utils::str_to_pginterval(&interval.unwrap_or_default())
            .expect("Invalid interval");

        db.activities()
            .fetch_top_buyers(&id, i)
            .await
            .expect("Failed to fetch collection top buyers")
    }

    async fn collection_top_seller(
        &self,
        ctx: &Context<'_>,
        id: String,
        interval: Option<String>,
    ) -> Vec<TopSellerSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let i = string_utils::str_to_pginterval(&interval.unwrap_or_default())
            .expect("Invalid interval");

        db.activities()
            .fetch_top_sellers(&id, i)
            .await
            .expect("Failed to fetch collection top sellers")
    }

    async fn collection_nft_holders(
        &self,
        ctx: &Context<'_>,
        id: String,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<NftHolderSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.nfts()
            .fetch_nft_holders(&id, limit, offset)
            .await
            .expect("Failed to fetch nft holders")
    }

    async fn collection_nft_amount_distribution(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Option<NftAmountDistributionSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.nfts().fetch_nft_amount_distribution(&id).await.ok()
    }

    async fn collection_nft_period_distribution(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Option<NftPeriodDistributionSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.nfts().fetch_nft_period_distribution(&id).await.ok()
    }

    async fn collection_floor_chart(
        &self,
        ctx: &Context<'_>,
        id: String,
        start_time: i64,
        end_time: i64,
        interval: String,
    ) -> Vec<DataPointSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let i = string_utils::str_to_pginterval(&interval)
            .expect("Invalid interval")
            .expect("Invalid interval");

        let start_date = DateTime::from_timestamp_millis(start_time).expect("Invalid start time");
        let end_date = DateTime::from_timestamp_millis(end_time).expect("Invalid end time");

        db.activities()
            .fetch_floor_chart(&id, start_date, end_date, i)
            .await
            .expect("Failed to fetch floor chart")
    }

    async fn activity_contribution_chart(
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
