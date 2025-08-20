use axum::{
    Extension,
    extract::{Json, Path, State},
    response::{IntoResponse, Response},
};
use sqlx::postgres::types::PgInterval;

use crate::{
    cache::ICache,
    database::{IDatabase, request_logs::IRequestLogs},
    http_server::{
        controllers::InternalState,
        middlewares::authentication::Claims,
        utils::{err_handler::response_400_with_const, validator::QueryValidator},
    },
    models::{api::requests::time_range::TimeRange, schema::data_point::DataPointSchema},
};

pub const REQUEST_LOG_TAG: &str = "request-log";

#[utoipa::path(
  get,
  path = "/users",
  tag = REQUEST_LOG_TAG,
  params(
      ("startTime" = Option<i64>, Query),
      ("endTime" = Option<i64>, Query),
      ("interval" = Option<String>, Query)
  ),
  responses(
    (status = 200, description = "Returns a list of user logs", body = [DataPointSchema])
  ),
  security(
    ("BearerAuth" = [])
  )
)]
pub async fn fetch_user_logs<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    QueryValidator(query): QueryValidator<TimeRange>,
    Extension(claims): Extension<Claims>,
) -> Response {
    let interval = PgInterval {
        months: query.interval.months,
        days: query.interval.days,
        microseconds: query.interval.microseconds,
    };

    match state
        .db
        .request_logs()
        .fetch_user_logs(&claims.id, query.start_time, query.end_time, interval)
        .await
    {
        Ok(data) => Json(data).into_response(),
        Err(_) => response_400_with_const(),
    }
}

#[utoipa::path(
  get,
  path = "/api-keys/{id}",
  tag = REQUEST_LOG_TAG,
  params(
      ("id" = String, Path, description = "Api key id"),
      ("startTime" = Option<i64>, Query),
      ("endTime" = Option<i64>, Query),
      ("interval" = Option<String>, Query)
  ),
  responses(
    (status = 200, description = "Returns a list of user api key logs", body = [DataPointSchema])
  ),
  security(
    ("BearerAuth" = [])
  )
)]
pub async fn fetch_api_key_logs<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<TimeRange>,
    Extension(claims): Extension<Claims>,
) -> Response {
    let interval = PgInterval {
        months: query.interval.months,
        days: query.interval.days,
        microseconds: query.interval.microseconds,
    };

    match state
        .db
        .request_logs()
        .fetch_api_key_logs(&claims.id, &id, query.start_time, query.end_time, interval)
        .await
    {
        Ok(data) => Json(data).into_response(),
        Err(_) => response_400_with_const(),
    }
}
