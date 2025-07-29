use std::sync::Arc;

use async_graphql::{Context, Object, http::GraphiQLSource};
use axum::response::{Html, IntoResponse};

use crate::{
    database::{Database, IDatabase, activities::IActivities, collections::ICollections},
    models::api::responses::{activity::Activity, collection::Collection},
    utils::string_utils,
};

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
            .expect("Failed to fetch collections")
    }

    async fn collections(
        &self,
        ctx: &Context<'_>,
        id: Option<String>,
        interval: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<Collection> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let (interval, limit, offset) = if let Some(collection_id) = id.as_ref() {
            if collection_id == "" {
                (None, 0, 0)
            } else {
                (None, 1, 0)
            }
        } else {
            let i = string_utils::str_to_pginterval(&interval.unwrap_or_default())
                .expect("Invalid interval");

            (i, limit.unwrap_or(10), offset.unwrap_or(0))
        };

        if limit == 0 {
            return Vec::new();
        }

        db.collections()
            .fetch_collections(id, interval, limit, offset)
            .await
            .expect("Failed to fetch collections")
    }
}
