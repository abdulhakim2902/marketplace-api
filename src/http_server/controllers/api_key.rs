use crate::{
    cache::ICache,
    database::{IDatabase, api_keys::IApiKeys},
    http_server::{
        controllers::InternalState,
        middlewares::authentication::Claims,
        utils::err_handler::{
            response_404_unhandled_err, response_404_with_message, response_429_unhandled_err,
        },
    },
    models::{
        api::{
            requests::{create_api_key::CreateApiKey, update_api_key::UpdateApiKey},
            responses::api_key::{ApiKeyResponse, SuccessApiKeyResponse},
        },
        db::api_key::DbApiKey,
    },
};
use axum::{
    Extension,
    extract::{Json, Path, State},
    response::{IntoResponse, Response},
};

pub const USER_TAG: &str = "user";

#[utoipa::path(
    get,
    path = "/api-keys",
    tag = USER_TAG,
    responses(
        (status = 200, description = "Returns a list of user api keys", body = [DbApiKey])
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn fetch_api_keys<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Extension(claims): Extension<Claims>,
) -> Response {
    match state.db.api_keys().fetch_api_keys(&claims.id).await {
        Ok(data) => Json(data).into_response(),
        Err(e) => response_404_unhandled_err(e),
    }
}

#[utoipa::path(
    post,
    path = "/api-keys",
    tag = USER_TAG,
    responses(
        (status = 200, description = "Returns a new created api key", body = [ApiKeyResponse])
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn create_api_key<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateApiKey>,
) -> Response {
    match state
        .db
        .api_keys()
        .create_api_key(
            &claims.id,
            &req.name,
            req.description.as_ref().map(|e| e.as_str()),
        )
        .await
    {
        Ok((id, key, created_at)) => Json(ApiKeyResponse {
            id,
            user_id: claims.id,
            name: req.name,
            description: req.description,
            key,
            created_at,
        })
        .into_response(),
        Err(e) => response_429_unhandled_err(e),
    }
}

#[utoipa::path(
    patch,
    path = "/api-keys/{id}",
    tag = USER_TAG,
    params(
        ("id" = String, Path, description = "Api key id")
    ),
    responses(
        (status = 200, description = "Returns a successful message", body = SuccessApiKeyResponse)
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn update_api_key<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateApiKey>,
) -> Response {
    match state
        .db
        .api_keys()
        .update_api_key(&id, &claims.id, &req)
        .await
    {
        Ok(res) => {
            if res.rows_affected() <= 0 {
                response_404_with_message("Api key not found")
            } else {
                Json(SuccessApiKeyResponse {
                    id,
                    message: "Successfully update api key".to_string(),
                })
                .into_response()
            }
        }
        Err(e) => response_429_unhandled_err(e),
    }
}

#[utoipa::path(
    delete,
    path = "/api-keys/{id}",
    tag = USER_TAG,
    params(
        ("id" = String, Path, description = "Api key id")
    ),
    responses(
        (status = 200, description = "Returns a successful message", body = SuccessApiKeyResponse)
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn remove_api_key<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Path(id): Path<String>,
    Extension(claims): Extension<Claims>,
) -> Response {
    match state.db.api_keys().remove_api_key(&id, &claims.id).await {
        Ok(res) => {
            if res.rows_affected() <= 0 {
                response_404_with_message("Api key not found")
            } else {
                Json(SuccessApiKeyResponse {
                    id,
                    message: "Successfully remove api key".to_string(),
                })
                .into_response()
            }
        }
        Err(e) => response_429_unhandled_err(e),
    }
}
