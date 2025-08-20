use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use jsonwebtoken::errors::Error;
use validator::ValidationErrors;

use crate::models::api::responses::{
    BAD_REQUEST_ERR, HttpResponse, HttpResponseErr, UNAUTHORIZED_ERR,
};

pub fn response_400_with_const() -> Response {
    (StatusCode::BAD_REQUEST, Json(BAD_REQUEST_ERR)).into_response()
}

pub fn response_400_with_message(msg: &str) -> Response {
    let error = HttpResponseErr::new("ERR_400", msg);

    (StatusCode::BAD_REQUEST, Json(error)).into_response()
}

pub fn response_401_with_const() -> Response {
    (StatusCode::UNAUTHORIZED, Json(UNAUTHORIZED_ERR)).into_response()
}

pub fn response_401_with_message(msg: &str) -> Response {
    let error = HttpResponseErr::new("ERR_401", msg);

    (StatusCode::UNAUTHORIZED, Json(error)).into_response()
}

pub fn response_404_with_message(msg: &str) -> Response {
    let error = HttpResponseErr::new("ERR_404", msg);

    (StatusCode::NOT_FOUND, Json(error)).into_response()
}

pub fn response_401_unhandled_err(e: Error) -> Response {
    let error = HttpResponseErr::new("ERR_401", &e.to_string());

    (StatusCode::UNAUTHORIZED, Json(error)).into_response()
}

pub fn response_404_unhandled_err(e: anyhow::Error) -> Response {
    let error = HttpResponseErr::new("ERR_404", &e.to_string());

    (StatusCode::BAD_REQUEST, Json(error)).into_response()
}

pub fn response_429_unhandled_err(e: anyhow::Error) -> Response {
    let error = HttpResponseErr::new("ERR_429", &e.to_string());

    (StatusCode::UNPROCESSABLE_ENTITY, Json(error)).into_response()
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
