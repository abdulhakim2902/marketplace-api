use std::sync::Arc;

use crate::{
    database::{
        Database, IDatabase, activities::IActivities, bids::IBids, collections::ICollections,
        listings::IListings, nfts::INfts,
    },
    models::schema::{
        activity::{ActivitySchema, FilterActivitySchema},
        bid::{BidSchema, FilterBidSchema},
        collection::{CollectionSchema, FilterCollectionSchema},
        collection_trending::CollectionTrendingSchema,
        data_point::DataPointSchema,
        listing::{FilterListingSchema, ListingSchema},
        nft::{FilterNftSchema, NftSchema},
        nft_change::NftChangeSchema,
        nft_distribution::{NftAmountDistributionSchema, NftPeriodDistributionSchema},
        nft_holder::NftHolderSchema,
        profit_leaderboard::ProfitLeaderboardSchema,
        top_buyer::TopBuyerSchema,
        top_seller::TopSellerSchema,
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
            .fetch_activities(query.collection_id, query.nft_id, limit, offset)
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
            .fetch_collections(query.collection_id, limit, offset)
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
            .fetch_nfts(query.nft_id, query.collection_id, limit, offset)
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
            .fetch_listings(query.nft_id, query.is_listed, limit, offset)
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
            .fetch_bids(query.collection_id, query.nft_id, limit, offset)
            .await
            .expect("Failed to fetch bids")
    }

    async fn collection_trending(
        &self,
        ctx: &Context<'_>,
        id: String,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<CollectionTrendingSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.collections()
            .fetch_collection_trending(&id, limit, offset)
            .await
            .expect("Failed to fetch collection trending")
    }

    async fn collection_top_buyer(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
        interval: Option<String>,
    ) -> Vec<TopBuyerSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let i = string_utils::str_to_pginterval(&interval.unwrap_or_default())
            .expect("Invalid interval");

        db.activities()
            .fetch_top_buyers(&collection_id, i)
            .await
            .expect("Failed to fetch collection top buyers")
    }

    async fn collection_top_seller(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
        interval: Option<String>,
    ) -> Vec<TopSellerSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let i = string_utils::str_to_pginterval(&interval.unwrap_or_default())
            .expect("Invalid interval");

        db.activities()
            .fetch_top_sellers(&collection_id, i)
            .await
            .expect("Failed to fetch collection top sellers")
    }

    async fn collection_nft_holders(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<NftHolderSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.nfts()
            .fetch_nft_holders(&collection_id, limit, offset)
            .await
            .expect("Failed to fetch nft holders")
    }

    async fn collection_nft_amount_distribution(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
    ) -> NftAmountDistributionSchema {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.nfts()
            .fetch_nft_amount_distribution(&collection_id)
            .await
            .expect("Failed to fetch collection nft amount distribution")
    }

    async fn collection_nft_period_distribution(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
    ) -> NftPeriodDistributionSchema {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.nfts()
            .fetch_nft_period_distribution(&collection_id)
            .await
            .expect("Failed to fetch collection nft period distribution")
    }

    async fn collection_profit_leaderboard(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<ProfitLeaderboardSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.activities()
            .fetch_profit_leaderboard(&collection_id, limit, offset)
            .await
            .expect("Failed to fetch collection nft period distribution")
    }

    async fn collection_nft_changes(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
        interval: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<NftChangeSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let i = string_utils::str_to_pginterval(&interval.unwrap_or_default())
            .expect("Invalid interval");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.activities()
            .fetch_nft_changes(&collection_id, i, limit, offset)
            .await
            .expect("Failed to fetch collection nft period distribution")
    }

    async fn collection_floor_chart(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
        start_time_in_ms: i64,
        end_time_in_ms: i64,
        interval: String,
    ) -> Vec<DataPointSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let i = string_utils::str_to_pginterval(&interval)
            .expect("Invalid interval")
            .expect("Invalid interval");

        let start_date =
            DateTime::from_timestamp_millis(start_time_in_ms).expect("Invalid start time");
        let end_date = DateTime::from_timestamp_millis(end_time_in_ms).expect("Invalid end time");

        db.activities()
            .fetch_floor_chart(&collection_id, start_date, end_date, i)
            .await
            .expect("Failed to fetch nfts")
    }
}
