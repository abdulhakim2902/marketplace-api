use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use jsonwebtoken::errors::Error;

use crate::models::api::responses::{HttpResponseErr, UNAUTHORIZED_ERR};

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
