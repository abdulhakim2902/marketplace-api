use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use validator::Validate;

use crate::{
    cache::ICache,
    database::{IDatabase, request_logs::IRequestLogs, users::IUsers},
    http_server::{
        controllers::InternalState,
        utils::{
            err_handler::{
                response_400_with_const, response_400_with_message, response_404_unhandled_err,
                response_404_with_message, response_429_unhandled_err,
            },
            validator::QueryValidator,
        },
    },
    models::api::{
        requests::{
            create_user::CreateUser, time_range::SummaryTimeRange, update_user::UpdateUser,
        },
        responses::{
            api_key::SuccessApiKeyResponse, log::UserLogSummaryResponse, user::UserResponse,
        },
    },
};

pub const ADMIN_TAG: &str = "admin";

#[utoipa::path(
    get,
    path = "/user",
    tag = ADMIN_TAG,
    responses(
        (status = 200, description = "Returns a list of users", body = [UserResponse])
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn fetch_user<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
) -> Response {
    match state.db.users().fetch_users().await {
        Ok(data) => Json(data).into_response(),
        Err(e) => response_404_unhandled_err(e),
    }
}

#[utoipa::path(
    post,
    path = "/user",
    tag = ADMIN_TAG,
    responses(
        (status = 200, description = "Returns a new created user", body = [UserResponse])
    ),
    security(
        ("BearerAuth" = [])
    )
)]
pub async fn create_user<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Json(req): Json<CreateUser>,
) -> Response {
    if let Err(e) = req.validate() {
        return response_400_with_message(&e.to_string());
    }

    match state.db.users().create_user(&req).await {
        Ok((id, created_at)) => Json(UserResponse {
            id,
            username: req.username,
            billing: Some(req.billing),
            active: true,
            created_at: created_at,
        })
        .into_response(),
        Err(e) => response_429_unhandled_err(e),
    }
}

#[utoipa::path(
    patch,
    path = "/user/{id}",
    tag = ADMIN_TAG,
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
pub async fn update_user<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Path(id): Path<String>,
    Json(req): Json<UpdateUser>,
) -> Response {
    if let Err(e) = req.validate() {
        return response_400_with_message(&e.to_string());
    }

    match state.db.users().update_user(&id, &req).await {
        Ok(res) => {
            if res.rows_affected() <= 0 {
                response_404_with_message("User not found")
            } else {
                Json(SuccessApiKeyResponse {
                    id,
                    message: "Successfully update user".to_string(),
                })
                .into_response()
            }
        }
        Err(e) => response_429_unhandled_err(e),
    }
}

#[utoipa::path(
  get,
  path = "/user/{id}/logs/summaries",
  tag = ADMIN_TAG,
  params(
      ("id" = String, Path),
      ("startTime" = Option<i64>, Query),
      ("endTime" = Option<i64>, Query)
  ),
  responses(
    (status = 200, description = "Returns a list of user log summary", body = [UserLogSummaryResponse])
  ),
  security(
    ("BearerAuth" = [])
  )
)]
pub async fn fetch_user_summaries<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<SummaryTimeRange>,
) -> Response {
    match state
        .db
        .request_logs()
        .fetch_summaries(&id, query.start_time, query.end_time)
        .await
    {
        Ok(data) => Json(data).into_response(),
        Err(_) => response_400_with_const(),
    }
}
