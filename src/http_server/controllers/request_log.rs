use axum::{
    Extension,
    extract::{Json, State},
    response::{IntoResponse, Response},
};

use crate::{
    cache::ICache,
    database::{IDatabase, request_logs::IRequestLogs},
    http_server::{
        controllers::{InternalState, api_key::USER_TAG},
        middlewares::authentication::Claims,
        utils::{err_handler::response_400_with_const, validator::QueryValidator},
    },
    models::{
        api::{
            requests::time_range::{SummaryTimeRange, TimeRange},
            responses::log::UserLogSummaryResponse,
        },
        schema::data_point::DataPointSchema,
    },
};

#[utoipa::path(
  get,
  path = "/logs/chart",
  tag = USER_TAG,
  params(
      ("apiKeyId" = Option<String>, Query),
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
pub async fn fetch_logs<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    QueryValidator(query): QueryValidator<TimeRange>,
    Extension(claims): Extension<Claims>,
) -> Response {
    match state
        .db
        .request_logs()
        .fetch_logs(
            &claims.id,
            query.api_key_id.as_ref().map(|e| e.as_str()),
            query.start_time,
            query.end_time,
            query.interval,
        )
        .await
    {
        Ok(data) => Json(data).into_response(),
        Err(_) => response_400_with_const(),
    }
}

#[utoipa::path(
  get,
  path = "/logs/summaries",
  tag = USER_TAG,
  params(
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
pub async fn fetch_summaries<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    QueryValidator(query): QueryValidator<SummaryTimeRange>,
    Extension(claims): Extension<Claims>,
) -> Response {
    match state
        .db
        .request_logs()
        .fetch_summaries(&claims.id, query.start_time, query.end_time)
        .await
    {
        Ok(data) => Json(data).into_response(),
        Err(_) => response_400_with_const(),
    }
}
