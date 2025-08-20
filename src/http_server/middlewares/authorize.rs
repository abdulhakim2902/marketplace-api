use crate::http_server::{
    middlewares::authentication::Claims, utils::err_handler::response_401_with_message,
};
use axum::{extract::Request, middleware::Next, response::Response};

pub async fn authorize_admin(mut req: Request, next: Next) -> Result<Response, Response> {
    let _ = req
        .extensions_mut()
        .get::<Claims>()
        .filter(|claim| claim.role == "admin")
        .ok_or(response_401_with_message("Only for admin"))?;

    Ok(next.run(req).await)
}

pub async fn authorize_user(mut req: Request, next: Next) -> Result<Response, Response> {
    let _ = req
        .extensions_mut()
        .get::<Claims>()
        .filter(|claim| claim.role == "user")
        .ok_or(response_401_with_message("Only for user"))?;

    Ok(next.run(req).await)
}
