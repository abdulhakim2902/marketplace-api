use std::sync::Arc;

use async_graphql::{Context, Object, http::GraphiQLSource};
use axum::response::{Html, IntoResponse};

use crate::{
    database::{Database, IDatabase, collections::ICollections},
    models::api::responses::{collection::Collection, collection_nft::CollectionNft},
    utils::string_utils,
};

pub async fn graphql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/gql").finish())
}
pub struct Query;

#[Object]
impl Query {
    async fn collections(
        &self,
        ctx: &Context<'_>,
        interval: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<Collection> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing service in the context");

        let interval = interval.unwrap_or_default();
        let pg_interval =
            string_utils::str_to_pginterval(&interval).expect("Invalid interval format");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        let collections = db
            .collections()
            .filter(pg_interval, limit, offset)
            .await
            .expect("Failed to fetch collections");

        collections
    }

    async fn collection_nfts(
        &self,
        ctx: &Context<'_>,
        id: String,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<CollectionNft> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing service in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        let nfts = db
            .collections()
            .fetch_collection_nfts(&id, limit, offset)
            .await
            .expect("Failed to fetch collections");

        nfts
    }
}
