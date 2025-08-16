use axum::{
    extract::{Json, State},
    response::{IntoResponse, Response},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header};

use crate::{
    cache::ICache,
    http_server::{
        middlewares::authentication::Claims,
        utils::err_handler::{
            response_400_with_const, response_401_with_const, response_429_unhandled_err,
        },
    },
    models::api::{requests::login::Login, responses::access_token::AccessToken},
};
use crate::{
    database::{IDatabase, users::IUsers},
    http_server::controllers::InternalState,
};

pub async fn login<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Json(req): Json<Login>,
) -> Response {
    match state.db.users().fetch_user_by_username(&req.username).await {
        Ok(res) => {
            let is_verified = bcrypt::verify(&req.password, &res.password).unwrap_or(false);

            if is_verified {
                let (value_str, unit) = state
                    .config
                    .jwt_config
                    .expires_in
                    .as_ref()
                    .map(|e| e.split_at(e.len() - 1))
                    .unwrap_or(("30", "d"));

                let value: i64 = value_str.parse().unwrap_or(1);
                let duration = match unit {
                    "s" => Duration::seconds(value),
                    "m" => Duration::minutes(value),
                    "h" => Duration::hours(value),
                    "d" => Duration::days(value),
                    _ => Duration::days(30),
                };

                let claims = Claims {
                    id: res.id.to_string(),
                    role: res.role,
                    exp: (Utc::now().timestamp() + duration.num_seconds()) as usize,
                    iat: Utc::now().timestamp() as usize,
                };

                let token_result = jsonwebtoken::encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(state.config.jwt_config.secret.as_ref()),
                );

                match token_result {
                    Ok(token) => Json(AccessToken { token }).into_response(),
                    Err(_) => response_400_with_const(),
                }
            } else {
                response_401_with_const()
            }
        }
        Err(e) => response_429_unhandled_err(e),
    }
}
