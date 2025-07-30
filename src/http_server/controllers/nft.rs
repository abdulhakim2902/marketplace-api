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
    models::api::{requests::filter_offer::FilterOffer, responses::HttpResponsePaging},
    services::{IInternalServices, nft::INftService},
};

pub async fn offers<TDb: IDatabase, TInternalService: IInternalServices>(
    State(state): InternalState<TDb, TInternalService>,
    Path(id): Path<String>,
    QueryValidator(query): QueryValidator<FilterOffer>,
) -> Response {
    match state
        .services
        .nft_service
        .fetch_nft_offers(&id, &query)
        .await
    {
        Ok((data, total)) => Json(HttpResponsePaging { data, total }).into_response(),
        Err(err) => response_unhandled_err(err),
    }
}
