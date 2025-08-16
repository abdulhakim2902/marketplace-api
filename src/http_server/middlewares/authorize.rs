use crate::http_server::utils::err_handler::response_401_with_message;
use axum::{extract::Request, middleware::Next, response::Response};

pub async fn authorize(mut req: Request, next: Next) -> Result<Response, Response> {
    let _ = req
        .extensions_mut()
        .get::<(String, String)>()
        .filter(|(_, role)| role == "admin")
        .ok_or(response_401_with_message("Only for admin"))?;

    Ok(next.run(req).await)
}
