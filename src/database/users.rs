use std::{str::FromStr, sync::Arc};

use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::api::{requests::create_user::CreateUser, responses::user::UserResponse};

#[async_trait::async_trait]
pub trait IUsers: Send + Sync {
    async fn fetch_users(&self) -> anyhow::Result<Vec<UserResponse>>;

    async fn create_user(&self, data: &CreateUser) -> anyhow::Result<(Uuid, DateTime<Utc>)>;

    async fn is_valid_user(&self, id: &str, role: &str) -> anyhow::Result<bool>;
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
    async fn fetch_users(&self) -> anyhow::Result<Vec<UserResponse>> {
        let res = sqlx::query_as!(
            UserResponse,
            r#"
            SELECT
                u.id,
                u.username,
                u.role,
                u.billing,
                u.active,
                u.created_at
            FROM users u
            WHERE u.role = 'user'
            "#
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch users")?;

        Ok(res)
    }

    async fn create_user(&self, data: &CreateUser) -> anyhow::Result<(Uuid, DateTime<Utc>)> {
        let password = data.password().context("Failed to hash password")?;
        let res = sqlx::query!(
            r#"
            INSERT INTO users (username, password, role, billing)
            VALUES ($1, $2, 'user', $3)
            RETURNING id, created_at;
            "#,
            data.username,
            password,
            data.billing,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to insert user")?;

        Ok((res.id, res.created_at))
    }

    async fn is_valid_user(&self, id: &str, role: &str) -> anyhow::Result<bool> {
        let res = sqlx::query!(
            r#"
            SELECT * FROM users u
            WHERE u.id = $1 AND u.role = $2
            LIMIT 1
            "#,
            Uuid::from_str(id).ok(),
            role,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch user")?;

        Ok(!res.is_empty())
    }
}
