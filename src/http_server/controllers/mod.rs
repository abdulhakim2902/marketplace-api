use crate::{cache::ICache, database::IDatabase, http_server::HttpServer};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::http::HeaderMap;
use std::sync::Arc;

pub mod health;

type InternalState<TDb, TCache> = State<Arc<HttpServer<TDb, TCache>>>;

#[derive(Clone, Debug)]
pub struct ApiKey {
    pub user: String,
    pub key: String,
}

pub async fn graphql_handler<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();

    let result = headers
        .get("x-api-user")
        .zip(headers.get("x-api-key"))
        .map(|(user, key)| user.to_str().ok().zip(key.to_str().ok()))
        .flatten();

    if let Some((api_user, api_key)) = result {
        req = req.data(ApiKey {
            user: api_user.to_owned(),
            key: api_key.to_owned(),
        });
    }

    state.schema.execute(req).await.into()
}
