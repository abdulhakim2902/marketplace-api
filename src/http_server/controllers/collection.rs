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
            filter_activity::FilterActivity, filter_collection::FilterCollection,
            filter_nft::FilterNft, filter_nft_change::FilterNftChange,
            filter_nft_holder::FilterNftHolder, filter_nft_trending::FilterNftTrending,
            filter_offer::FilterOffer, filter_profit_leaderboard::FilterProfitLeader,
            filter_top_buyer::FilterTopBuyer, filter_top_seller::FilterTopSeller,
            floor_chart::FloorChart,
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
        Ok((data, total)) => Json(HttpResponsePaging { data, total }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}

pub async fn activities<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<FilterActivity>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_activities(&id, &query)
        .await
    {
        Ok((data, total)) => Json(HttpResponsePaging { data, total }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}

pub async fn floor_chart<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<FloorChart>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_floor_chart(&id, &query)
        .await
    {
        Ok(data) => Json(HttpResponse { data }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}

pub async fn top_buyers<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<FilterTopBuyer>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_top_buyer(&id, &query)
        .await
    {
        Ok(data) => Json(HttpResponse { data }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}

pub async fn top_sellers<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<FilterTopSeller>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_top_seller(&id, &query)
        .await
    {
        Ok(data) => Json(HttpResponse { data }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}

pub async fn nft_holders<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<FilterNftHolder>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_nft_holders(&id, &query)
        .await
    {
        Ok((data, total)) => Json(HttpResponsePaging { data, total }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}

pub async fn nft_trendings<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<FilterNftTrending>,
) -> Response {
    match state
        .services
        .collection_service
        .fetch_collection_trending_nfts(&id, &query)
        .await
    {
        Ok((data, total)) => Json(HttpResponsePaging { data, total }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}

pub async fn nft_amount_distribution<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
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

pub async fn nft_period_distribution<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
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

pub async fn profit_leaderboard<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
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

pub async fn nft_change<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
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
