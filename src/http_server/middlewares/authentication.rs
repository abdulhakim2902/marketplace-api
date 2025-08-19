use std::sync::Arc;

use crate::{
    database::{IDatabase, users::IUsers},
    http_server::utils::err_handler::{
        response_401_unhandled_err, response_401_with_const, response_401_with_message,
    },
};
use axum::{extract::Request, http::header, middleware::Next, response::Response};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub id: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

pub async fn authentication<TDb: IDatabase>(
    mut req: Request,
    next: Next,
    db: Arc<TDb>,
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

    let _ = db
        .users()
        .is_valid_user(&token_data.claims.id, &token_data.claims.role)
        .await
        .ok()
        .filter(|e| *e)
        .ok_or(response_401_with_message("User not found"))?;

    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}
