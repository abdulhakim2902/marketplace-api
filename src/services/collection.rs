use std::sync::Arc;

use crate::{
    database::{IDatabase, collections::ICollections},
    models::api::{
        requests::{
            filter_activity::FilterActivity, filter_collection::FilterCollection,
            filter_nft::FilterNft, filter_offer::FilterOffer,
        },
        responses::{
            collection::Collection, collection_activity::CollectionActivity,
            collection_info::CollectionInfo, collection_nft::CollectionNft,
        },
    },
};

#[async_trait::async_trait]
pub trait ICollectionService {
    async fn fetch_collections(
        &self,
        filter: &FilterCollection,
    ) -> anyhow::Result<(Vec<Collection>, i64)>;

    async fn fetch_collection_info(&self, id: &str) -> anyhow::Result<CollectionInfo>;

    async fn fetch_collection_nfts(
        &self,
        id: &str,
        filter: &FilterNft,
    ) -> anyhow::Result<(Vec<CollectionNft>, i64)>;

    async fn fetch_collection_offers(&self, id: &str, filter: &FilterOffer) -> anyhow::Result<()>;

    async fn fetch_collection_activities(
        &self,
        id: &str,
        filter: &FilterActivity,
    ) -> anyhow::Result<(Vec<CollectionActivity>, i64)>;
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

    async fn fetch_collection_nfts(
        &self,
        id: &str,
        filter: &FilterNft,
    ) -> anyhow::Result<(Vec<CollectionNft>, i64)> {
        let repository = self.db.collections();

        let filter_fut =
            repository.fetch_collection_nfts(id, filter.paging.page, filter.paging.page_size);

        let count_fut = repository.count_collection_nfts(&id);

        let (data_res, count_res) = tokio::join!(filter_fut, count_fut);
        let (data, count) = (data_res?, count_res?);

        Ok((data, count))
    }

    async fn fetch_collection_offers(
        &self,
        _id: &str,
        _filter: &FilterOffer,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn fetch_collection_activities(
        &self,
        id: &str,
        filter: &FilterActivity,
    ) -> anyhow::Result<(Vec<CollectionActivity>, i64)> {
        let repository = self.db.collections();

        let filter_fut =
            repository.fetch_collection_activities(id, filter.paging.page, filter.paging.page_size);

        let count_fut = repository.count_collection_activities(&id);

        let (data_res, count_res) = tokio::join!(filter_fut, count_fut);
        let (data, count) = (data_res?, count_res?);

        Ok((data, count))
    }
}
