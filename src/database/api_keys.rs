use anyhow::Context;
use chrono::{DateTime, Utc};
use prefixed_api_key::PrefixedApiKeyController;
use sqlx::PgPool;
use sqlx::postgres::PgQueryResult;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::api::requests::update_api_key::UpdateApiKey;
use crate::models::db::api_key::DbApiKey;

#[async_trait::async_trait]
pub trait IApiKeys: Send + Sync {
    async fn create_api_key(
        &self,
        user_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> anyhow::Result<(Uuid, String, DateTime<Utc>)>;

    async fn fetch_api_keys(&self, user_id: &str) -> anyhow::Result<Vec<DbApiKey>>;

    async fn update_api_key(
        &self,
        id: &str,
        user_id: &str,
        data: &UpdateApiKey,
    ) -> anyhow::Result<PgQueryResult>;

    async fn remove_api_key(&self, id: &str, user_id: &str) -> anyhow::Result<PgQueryResult>;

    async fn is_valid_api_key(
        &self,
        username: &str,
        token: Option<String>,
    ) -> anyhow::Result<(Uuid, Uuid, String, bool)>;
}

pub struct ApiKeys {
    pool: Arc<PgPool>,
}

impl ApiKeys {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl IApiKeys for ApiKeys {
    async fn create_api_key(
        &self,
        user_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> anyhow::Result<(Uuid, String, DateTime<Utc>)> {
        let controller = PrefixedApiKeyController::configure()
            .prefix("ucc".to_owned())
            .seam_defaults()
            .finalize()
            .context("Failed building api key")?;

        let (key, hash) = controller.generate_key_and_hash();

        let short_token = key.short_token();

        let res = sqlx::query!(
            r#"
            INSERT INTO api_keys (user_id, name, description, short_token, long_token_hash)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (short_token)
              DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                updated_at = NOW()
            RETURNING id, created_at;
            "#,
            Uuid::from_str(user_id).ok(),
            name,
            description,
            short_token,
            hash,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to create api key")?;

        Ok((res.id, key.to_string(), res.created_at))
    }

    async fn fetch_api_keys(&self, user_id: &str) -> anyhow::Result<Vec<DbApiKey>> {
        let res = sqlx::query_as!(
            DbApiKey,
            r#"
            SELECT
              ak.id, 
              ak.user_id, 
              ak.name, 
              ak.description, 
              ak.created_at, 
              ak.updated_at 
            FROM api_keys ak
            WHERE ak.user_id = $1
            "#,
            Uuid::from_str(user_id).ok()
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch api keys")?;

        Ok(res)
    }

    async fn update_api_key(
        &self,
        id: &str,
        user_id: &str,
        data: &UpdateApiKey,
    ) -> anyhow::Result<PgQueryResult> {
        let res = sqlx::query!(
            r#"
            UPDATE api_keys ak1
            SET
                name = COALESCE($3, ak2.name),
                description = COALESCE($4, ak2.description),
                updated_at = NOW()
            FROM api_keys ak2
            WHERE ak1.id = $1
                AND ak1.user_id = $2
                AND ak1.id = ak2.id
                AND ak1.user_id = ak2.user_id
            "#,
            Uuid::from_str(id).ok(),
            Uuid::from_str(user_id).ok(),
            data.name,
            data.description,
        )
        .execute(&*self.pool)
        .await
        .context("Failed to update api key")?;

        Ok(res)
    }

    async fn remove_api_key(&self, id: &str, user_id: &str) -> anyhow::Result<PgQueryResult> {
        let res = sqlx::query!(
            r#"
            DELETE FROM api_keys ak
            WHERE ak.id = $1 AND ak.user_id = $2
            "#,
            Uuid::from_str(id).ok(),
            Uuid::from_str(user_id).ok(),
        )
        .execute(&*self.pool)
        .await
        .context("Failed to remove api keys")?;

        Ok(res)
    }

    async fn is_valid_api_key(
        &self,
        username: &str,
        token: Option<String>,
    ) -> anyhow::Result<(Uuid, Uuid, String, bool)> {
        let res = sqlx::query!(
            r#"
            SELECT ak.id, ak.long_token_hash, ak.user_id, u.active FROM users u
              JOIN api_keys ak ON ak.user_id = u.id
            WHERE u.username = $1
                AND ak.short_token = $2
                AND u.role = 'user'
            LIMIT 1
            "#,
            username,
            token
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to fetch")?;

        Ok((res.id, res.user_id, res.long_token_hash, res.active))
    }
}
