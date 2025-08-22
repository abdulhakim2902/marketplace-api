use crate::database::api_keys::IApiKeys;
use crate::database::request_logs::IRequestLogs;
use crate::{cache::ICache, database::IDatabase, http_server::HttpServer};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::http::HeaderMap;
use prefixed_api_key::PrefixedApiKey;
use std::sync::Arc;

pub mod api_key;
pub mod auth;
pub mod health;
pub mod request_log;
pub mod user;

type InternalState<TDb, TCache> = State<Arc<HttpServer<TDb, TCache>>>;

#[derive(Clone, Debug)]
pub struct ApiKey {
    pub user: String,
    pub key: String,
    pub token_hash: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct Origin(pub String);

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

        if let Some((id, user_id, hash, active)) = is_valid_api_key {
            let db = state.db.clone();

            tokio::spawn(async move {
                if let Err(e) = db.request_logs().add_logs(&id, &user_id).await {
                    tracing::error!("Failed to add logs: {e:#}")
                }
            });

            req = req.data(ApiKey {
                active,
                user: api_user.to_owned(),
                key: api_key.to_owned(),
                token_hash: hash,
            });
        }
    } else if let Some(origin) = headers.get("origin") {
        for allowed_origin in state.config.server_config.allowed_origins.iter() {
            let origin = origin
                .to_str()
                .ok()
                .filter(|o| o == allowed_origin)
                .map(|o| Origin(o.to_string()));

            if let Some(origin) = origin {
                req = req.data(origin);
                break;
            }
        }
    }

    state.schema.execute(req).await.into()
}
