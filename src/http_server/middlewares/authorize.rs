use crate::http_server::utils::err_handler::{response_401_unhandled_err, response_401_with_const};
use axum::{extract::Request, http::header, middleware::Next, response::Response};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub exp: usize,
    pub iat: usize,
}

pub async fn authorize(
    mut req: Request,
    next: Next,
    jwt_secret: String,
) -> Result<Response, Response> {
    let auth_header = req.headers_mut().get(header::AUTHORIZATION);
    let token = auth_header
        .and_then(|e| e.to_str().ok())
        .filter(|e| e.starts_with("Bearer"))
        .ok_or(response_401_with_const())?
        .split_whitespace()
        .last()
        .ok_or(response_401_with_const())?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| response_401_unhandled_err(e))?;

    req.extensions_mut().insert(token_data.claims.id);

    Ok(next.run(req).await)
}
