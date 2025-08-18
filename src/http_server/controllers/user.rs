use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};

use crate::{
    cache::ICache,
    database::users::IUsers,
    http_server::utils::err_handler::{response_404_unhandled_err, response_429_unhandled_err},
    models::api::{requests::create_user::CreateUser, responses::user::UserResponse},
};
use crate::{database::IDatabase, http_server::controllers::InternalState};

pub async fn fetch_user<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
) -> Response {
    match state.db.users().fetch_users().await {
        Ok(data) => Json(data).into_response(),
        Err(e) => response_404_unhandled_err(e),
    }
}

pub async fn create_user<TDb: IDatabase, TCache: ICache>(
    State(state): InternalState<TDb, TCache>,
    Json(req): Json<CreateUser>,
) -> Response {
    match state.db.users().create_user(&req).await {
        Ok((id, created_at)) => Json(UserResponse {
            id,
            username: req.username,
            role: "user".to_string(),
            billing: Some(req.billing),
            active: true,
            created_at: created_at,
        })
        .into_response(),
        Err(e) => response_429_unhandled_err(e),
    }
}
