use std::sync::Arc;

use axum::extract::State;

use crate::http_server::HttpServer;

pub mod collection;
pub mod health;

type InternalState<TInternalServices> = State<Arc<HttpServer<TInternalServices>>>;
