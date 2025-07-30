use std::sync::Arc;

use crate::{
    database::{
        Database, IDatabase, activities::IActivities, collections::ICollections, nfts::INfts,
    },
    models::api::responses::{
        activity::Activity, collection::Collection, data_point::DataPoint, nft::Nft,
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
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<Activity> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.activities()
            .fetch_activities(limit, offset)
            .await
            .expect("Failed to fetch activites")
    }

    async fn collections(
        &self,
        ctx: &Context<'_>,
        id: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<Collection> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.collections()
            .fetch_collections(id, limit, offset)
            .await
            .expect("Failed to fetch collections")
    }

    async fn nfts(
        &self,
        ctx: &Context<'_>,
        id: Option<String>,
        collection_id: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<Nft> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        db.nfts()
            .fetch_nfts(id, collection_id, limit, offset)
            .await
            .expect("Failed to fetch nfts")
    }

    async fn floor_chart(
        &self,
        ctx: &Context<'_>,
        collection_id: String,
        start_time_in_ms: i64,
        end_time_in_ms: i64,
        interval: String,
    ) -> Vec<DataPoint> {
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
