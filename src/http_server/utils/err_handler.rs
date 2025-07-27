use axum::{
    Json,
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;
use validator::ValidationErrors;

use crate::models::api::responses::{
    HttpResponse, HttpResponseConstErr, INTERNAL_SERVER_ERR, NOT_FOUND_ERR,
};

pub fn response_unhandled_err(e: anyhow::Error) -> Response {
    error!("Unhandled error: {:?}", e);
    (StatusCode::INTERNAL_SERVER_ERROR, Json(INTERNAL_SERVER_ERR)).into_response()
}

pub fn response_unhandled_str(e: &str) -> Response {
    error!("Unhandled error: {:?}", e);
    (StatusCode::INTERNAL_SERVER_ERROR, Json(INTERNAL_SERVER_ERR)).into_response()
}

pub fn _response_400_with_const(e: HttpResponseConstErr) -> Response {
    (StatusCode::BAD_REQUEST, Json(e)).into_response()
}

pub async fn response_404_err() -> Response {
    (StatusCode::NOT_FOUND, Json(NOT_FOUND_ERR)).into_response()
}

pub fn _response_cache_hit(value: Vec<u8>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(value))
        .expect("Failed to build response")
}

pub fn response_validation_err(e: ValidationErrors) -> (StatusCode, Json<HttpResponse<String>>) {
    let msg = e
        .field_errors()
        .iter()
        .map(|(field, errors)| {
            let error_messages: Vec<String> = errors
                .iter()
                .map(|error| format!("{}: {}", field, error.message.clone().unwrap_or_default()))
                .collect();
            error_messages.join(", ")
        })
        .collect::<Vec<String>>()
        .join("; ");

    (StatusCode::BAD_REQUEST, Json(HttpResponse { data: msg }))
}
