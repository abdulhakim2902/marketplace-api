use axum::{
    extract::State,
    http::{Response, StatusCode},
};

use crate::{
    http_server::controllers::InternalState,
    services::{IInternalServices, health::IHealthService},
};

pub async fn check<TInternalService: IInternalServices>(
    State(state): InternalState<TInternalService>,
) -> Response<String> {
    match state.services.health_service.is_healthy().await {
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
