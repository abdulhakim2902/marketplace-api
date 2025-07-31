use axum::{
    extract::State,
    http::{Response, StatusCode},
};

use crate::{database::IDatabase, http_server::controllers::InternalState};

pub async fn check<TDb: IDatabase>(State(state): InternalState<TDb>) -> Response<String> {
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
