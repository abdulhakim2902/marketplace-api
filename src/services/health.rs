use std::sync::Arc;

use crate::database::IDatabase;

#[async_trait::async_trait]
pub trait IHealthService {
    async fn is_healthy(&self) -> bool;
}

pub struct HealthService<TDb: IDatabase> {
    db: Arc<TDb>,
}

impl<TDb: IDatabase> HealthService<TDb> {
    pub fn new(db: Arc<TDb>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl<TDb> IHealthService for HealthService<TDb>
where
    TDb: IDatabase + Send + Sync + 'static,
{
    async fn is_healthy(&self) -> bool {
        self.db.is_healthy().await
    }
}
