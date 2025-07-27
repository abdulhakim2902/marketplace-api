use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};

use crate::{
    http_server::{
        controllers::InternalState,
        utils::{err_handler::response_unhandled_err, validator::QueryValidator},
    },
    models::api::{requests::filter_activity::FilterActivity, responses::HttpResponsePaging},
    services::{IInternalServices, nft::INftService},
};

pub async fn activities<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<FilterActivity>,
) -> Response {
    match state
        .services
        .nft_service
        .fetch_nft_activities(&id, &query)
        .await
    {
        Ok((data, total)) => Json(HttpResponsePaging { data, total }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}
