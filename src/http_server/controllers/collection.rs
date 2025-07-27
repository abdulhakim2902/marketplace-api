use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};

use crate::{
    http_server::{
        controllers::InternalState,
        utils::{err_handler::response_unhandled_err, validator::QueryValidator},
    },
    models::api::{requests::filter_collection::FilterCollection, responses::HttpResponsePaging},
    services::{IInternalServices, collection::ICollectionService},
};

pub async fn filter<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
    QueryValidator(query): QueryValidator<FilterCollection>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collections(&query)
        .await
    {
        Ok((data, total)) => Json(HttpResponsePaging { data, total }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}
