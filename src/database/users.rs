use std::sync::Arc;

use sqlx::PgPool;

#[async_trait::async_trait]
pub trait IUsers: Send + Sync {}

pub struct Users {
    _pool: Arc<PgPool>,
}

impl Users {
    pub fn new(_pool: Arc<PgPool>) -> Self {
        Self { _pool }
    }
}

#[async_trait::async_trait]
impl IUsers for Users {}
