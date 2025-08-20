use axum::{
    Json,
    extract::{FromRequestParts, Query},
    http::{StatusCode, request::Parts},
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::{
    http_server::utils::err_handler::response_validation_err, models::api::responses::HttpResponse,
};

pub struct QueryValidator<T>(pub T);

impl<S, T> FromRequestParts<S> for QueryValidator<T>
where
    S: Send + Sync,
    T: Validate + DeserializeOwned,
{
    type Rejection = (StatusCode, Json<HttpResponse<String>>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Query::<T>::from_request_parts(parts, state).await {
            Ok(Query(query)) => {
                query.validate().map_err(response_validation_err)?;

                Ok(Self(
                    serde_qs::from_str(parts.uri.query().unwrap_or_default()).unwrap(),
                ))
            }
            Err(rejection) => Err((
                rejection.status(),
                Json(HttpResponse {
                    data: rejection.body_text(),
                }),
            )),
        }
    }
}
