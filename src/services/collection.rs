use std::sync::Arc;

use crate::{
    database::{IDatabase, collections::ICollections},
    models::api::{
        requests::filter_collection::FilterCollection,
        responses::{collection::Collection, collection_info::CollectionInfo},
    },
};

#[async_trait::async_trait]
pub trait ICollectionService {
    async fn fetch_collections(
        &self,
        filter: &FilterCollection,
    ) -> anyhow::Result<(Vec<Collection>, i64)>;

    async fn fetch_collection_info(&self, id: &str) -> anyhow::Result<CollectionInfo>;
}

pub struct CollectionService<TDb: IDatabase> {
    db: Arc<TDb>,
}

impl<TDb: IDatabase> CollectionService<TDb> {
    pub fn new(db: Arc<TDb>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl<TDb> ICollectionService for CollectionService<TDb>
where
    TDb: IDatabase + Send + Sync + 'static,
{
    async fn fetch_collections(
        &self,
        filter: &FilterCollection,
    ) -> anyhow::Result<(Vec<Collection>, i64)> {
        let repository = self.db.collections();

        let filter_fut =
            repository.filter(filter.interval, filter.paging.page, filter.paging.page_size);
        let count_fut = repository.count();

        let (data_res, count_res) = tokio::join!(filter_fut, count_fut);
        let (data, count) = (data_res?, count_res?);

        Ok((data, count))
    }

    async fn fetch_collection_info(&self, id: &str) -> anyhow::Result<CollectionInfo> {
        self.db.collections().fetch_collection_info(id).await
    }
}
