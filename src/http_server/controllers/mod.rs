use std::sync::Arc;

use axum::extract::State;

use crate::http_server::HttpServer;

pub mod health;
pub mod nft;

type InternalState<TDb, TInternalServices> = State<Arc<HttpServer<TDb, TInternalServices>>>;
