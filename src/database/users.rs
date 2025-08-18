use std::{str::FromStr, sync::Arc};

use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, postgres::PgQueryResult};
use uuid::Uuid;

use crate::models::api::{
    requests::create_user::CreateUser,
    responses::{auth_user::AuthUserResponse, user::UserResponse},
};

#[async_trait::async_trait]
pub trait IUsers: Send + Sync {
    async fn fetch_user_by_username(&self, username: &str) -> anyhow::Result<AuthUserResponse>;

    async fn fetch_users(&self) -> anyhow::Result<Vec<UserResponse>>;

    async fn create_user(&self, data: &CreateUser) -> anyhow::Result<(Uuid, DateTime<Utc>)>;

    async fn create_admin_user(
        &self,
        username: &str,
        password: &str,
    ) -> anyhow::Result<PgQueryResult>;

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
    async fn fetch_user_by_username(&self, username: &str) -> anyhow::Result<AuthUserResponse> {
        let res = sqlx::query_as!(
            AuthUserResponse,
            r#"
            SELECT
                u.id,
                u.username,
                u.password,
                u.role
            FROM users u
            WHERE u.username = $1
            "#,
            username,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch user")?;

        Ok(res)
    }

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
            INSERT INTO users (username, password, billing, role)
            VALUES ($1, $2, $3, 'user')
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

    async fn create_admin_user(
        &self,
        username: &str,
        password: &str,
    ) -> anyhow::Result<PgQueryResult> {
        let password =
            bcrypt::hash(password, bcrypt::DEFAULT_COST).context("Failed to hash password")?;

        let res = sqlx::query!(
            r#"
            INSERT INTO users (username, password, role)
            VALUES ($1, $2, 'admin')
            ON CONFLICT (username) 
                DO UPDATE SET password = EXCLUDED.password;
            "#,
            username,
            password,
        )
        .execute(&*self.pool)
        .await
        .context("Failed to insert user")?;

        Ok(res)
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
