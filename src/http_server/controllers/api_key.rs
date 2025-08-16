use crate::{
    cache::ICache,
    database::{IDatabase, api_keys::IApiKeys},
    http_server::{
        controllers::InternalState,
        utils::err_handler::{
            response_404_unhandled_err, response_404_with_message, response_429_unhandled_err,
        },
    },
    models::api::{requests::create_api_key::CreateApiKey, responses::api_key::ApiKeyResponse},
};
use axum::{
    Extension,
    extract::{Json, Path, State},
    response::{IntoResponse, Response},
};

pub async fn fetch_api_keys<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Response {
    match state.db.api_keys().fetch_api_keys(&user_id).await {
        Ok(data) => Json(data).into_response(),
        Err(e) => response_404_unhandled_err(e),
    }
}

pub async fn create_api_key<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Extension((user_id, _)): Extension<(String, String)>,
    Json(req): Json<CreateApiKey>,
) -> Response {
    match state
        .db
        .api_keys()
        .create_api_key(
            &user_id,
            &req.name,
            req.description.as_ref().map(|e| e.as_str()),
        )
        .await
    {
        Ok((id, key, created_at)) => Json(ApiKeyResponse {
            id,
            user_id,
            name: req.name,
            description: req.description,
            key,
            created_at,
        })
        .into_response(),
        Err(e) => response_429_unhandled_err(e),
    }
}

pub async fn remove_api_key<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Path(id): Path<String>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Response {
    match state.db.api_keys().remove_api_key(&id, &user_id).await {
        Ok(res) => {
            if res.rows_affected() <= 0 {
                response_404_with_message("Api key not found")
            } else {
                Json(serde_json::json!({"id": id, "message": "Successfully remove api key"}))
                    .into_response()
            }
        }
        Err(e) => response_429_unhandled_err(e),
    }
}
