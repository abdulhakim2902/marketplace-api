use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};

use crate::{
    database::IDatabase,
    http_server::{
        controllers::InternalState,
        utils::{err_handler::response_unhandled_err, validator::QueryValidator},
    },
    models::api::{
        requests::{
            filter_nft_change::FilterNftChange, filter_offer::FilterOffer,
            filter_profit_leaderboard::FilterProfitLeader,
        },
        responses::{HttpResponse, HttpResponsePaging},
    },
    services::{IInternalServices, collection::ICollectionService},
};

pub async fn offers<TDb: IDatabase, TInternalService: IInternalServices>(
    State(state): InternalState<TDb, TInternalService>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<FilterOffer>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_offers(&id, &query)
        .await
    {
        Ok((data, total)) => Json(HttpResponsePaging { data, total }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}

pub async fn nft_amount_distribution<TDb: IDatabase, TInternalService: IInternalServices>(
    State(state): InternalState<TDb, TInternalService>,
    Path(id): Path<String>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_nft_amount_distribution(&id)
        .await
    {
        Ok(data) => Json(HttpResponse { data }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}

pub async fn nft_period_distribution<TDb: IDatabase, TInternalService: IInternalServices>(
    State(state): InternalState<TDb, TInternalService>,
    Path(id): Path<String>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_nft_period_distribution(&id)
        .await
    {
        Ok(data) => Json(HttpResponse { data }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}

pub async fn profit_leaderboard<TDb: IDatabase, TInternalService: IInternalServices>(
    State(state): InternalState<TDb, TInternalService>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<FilterProfitLeader>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_profit_leaderboard(&id, &query)
        .await
    {
        Ok((data, total)) => Json(HttpResponsePaging { data, total }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}

pub async fn nft_change<TDb: IDatabase, TInternalService: IInternalServices>(
    State(state): InternalState<TDb, TInternalService>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<FilterNftChange>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_nft_change(&id, &query)
        .await
    {
        Ok((data, total)) => Json(HttpResponsePaging { data, total }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}
