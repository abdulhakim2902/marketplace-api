use crate::database::request_logs::IRequestLogs;
use crate::database::users::IUsers;
use crate::{cache::ICache, database::IDatabase, http_server::HttpServer};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::http::HeaderMap;
use std::sync::Arc;
use uuid::Uuid;

pub mod health;

type InternalState<TDb, TCache> = State<Arc<HttpServer<TDb, TCache>>>;

#[derive(Clone, Debug)]
pub struct ApiKey {
    pub id: Uuid,
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
        let is_valid_api_key = state
            .db
            .users()
            .is_valid_api_key(api_user, api_key)
            .await
            .ok();

        if let Some(id) = is_valid_api_key {
            let db = state.db.clone();

            tokio::spawn(async move {
                if let Err(e) = db.request_logs().add_logs(&id).await {
                    tracing::error!("Failed to add logs: {e:#}")
                }
            });

            req = req.data(ApiKey {
                id,
                user: api_user.to_owned(),
                key: api_key.to_owned(),
            });
        }
    }

    state.schema.execute(req).await.into()
}
