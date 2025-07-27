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
    models::api::{
        requests::{
            filter_collection::FilterCollection, filter_nft::FilterNft, filter_offer::FilterOffer,
        },
        responses::{HttpResponse, HttpResponsePaging},
    },
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

pub async fn info<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
    Path(id): Path<String>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_info(&id)
        .await
    {
        Ok(data) => Json(HttpResponse { data }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}

pub async fn nfts<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<FilterNft>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_nfts(&id, &query)
        .await
    {
        Ok((data, total)) => Json(HttpResponsePaging { data, total }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}

pub async fn offers<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<FilterOffer>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_offers(&id, &query)
        .await
    {
        Ok(data) => Json(HttpResponse { data }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}
