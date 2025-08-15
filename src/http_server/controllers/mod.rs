use crate::database::api_keys::IApiKeys;
use crate::database::request_logs::IRequestLogs;
use crate::{cache::ICache, database::IDatabase, http_server::HttpServer};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::http::HeaderMap;
use prefixed_api_key::PrefixedApiKey;
use std::sync::Arc;

pub mod health;

type InternalState<TDb, TCache> = State<Arc<HttpServer<TDb, TCache>>>;

#[derive(Clone, Debug)]
pub struct ApiKey {
    pub user: String,
    pub key: String,
    pub token_hash: String,
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
        let token = PrefixedApiKey::from_string(api_key)
            .ok()
            .map(|e| e.short_token().to_string());

        let is_valid_api_key = state
            .db
            .api_keys()
            .is_valid_api_key(api_user, token)
            .await
            .ok();

        if let Some((id, hash)) = is_valid_api_key {
            let db = state.db.clone();

            tokio::spawn(async move {
                if let Err(e) = db.request_logs().add_logs(&id).await {
                    tracing::error!("Failed to add logs: {e:#}")
                }
            });

            req = req.data(ApiKey {
                user: api_user.to_owned(),
                key: api_key.to_owned(),
                token_hash: hash,
            });
        }
    }

    state.schema.execute(req).await.into()
}
