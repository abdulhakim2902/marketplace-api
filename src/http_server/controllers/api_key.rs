use crate::{
    cache::ICache,
    database::{IDatabase, api_keys::IApiKeys},
    http_server::controllers::InternalState,
};
use axum::{
    extract::{Path, State},
    http::{Response, StatusCode},
};

pub async fn fetch_api_keys<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
) -> Response<String> {
    // state.db.api_keys().fetch_api_keys().await;
    match state.db.is_healthy().await {
        true => Response::builder()
            .status(StatusCode::OK)
            .body("OK".into())
            .unwrap(),
        false => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Service is not healthy".into())
            .unwrap(),
    }
}

pub async fn create_api_key<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
) -> Response<String> {
    // state.db.api_keys().create_api_key().await;
    match state.db.is_healthy().await {
        true => Response::builder()
            .status(StatusCode::OK)
            .body("OK".into())
            .unwrap(),
        false => Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body("Service is not healthy".into())
            .unwrap(),
    }
}

pub async fn remove_api_key<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Path(id): Path<String>,
) -> Response<String> {
    match state.db.api_keys().remove_api_key(&id).await {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .body("OK".into())
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body("Failed to remove api key".into())
            .unwrap(),
    }
}
