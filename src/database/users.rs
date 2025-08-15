use anyhow::Context;
use std::sync::Arc;
use uuid::Uuid;

use sqlx::PgPool;

#[async_trait::async_trait]
pub trait IUsers: Send + Sync {
    async fn is_valid_api_key(&self, api_user: &str, api_key: &str) -> anyhow::Result<Uuid>;
}

pub struct Users {
    pool: Arc<PgPool>,
}

impl Users {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl IUsers for Users {
    async fn is_valid_api_key(&self, api_user: &str, api_key: &str) -> anyhow::Result<Uuid> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT ak.id FROM users u
              JOIN api_keys ak ON ak.user_id = u.id
            WHERE u.username = $1 AND ak.key = $2
            LIMIT 1
            "#,
            api_user,
            api_key
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch")?;

        Ok(res)
    }
}
