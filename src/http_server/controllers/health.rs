use axum::{
    extract::State,
    http::{Response, StatusCode},
};

use crate::cache::ICache;
use crate::{database::IDatabase, http_server::controllers::InternalState};

pub async fn check<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
) -> Response<String> {
    match state.db.is_healthy().await {
        true => Response::builder()
            .status(StatusCode::OK)
            .body("OK".into())
            .unwrap(),
        false => Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body("Service is not healthy".into())
            .unwrap(),
    }
}
