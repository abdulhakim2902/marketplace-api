use std::sync::Arc;

use crate::{
    database::{IDatabase, nfts::INfts},
    models::api::{
        requests::{filter_activity::FilterActivity, filter_listing::FilterListing},
        responses::{nft_activity::NftActivity, nft_info::NftInfo, nft_listing::NftListing},
    },
};

#[async_trait::async_trait]
pub trait INftService {
    async fn fetch_info(&self, id: &str) -> anyhow::Result<NftInfo>;

    async fn fetch_nft_activities(
        &self,
        id: &str,
        filter: &FilterActivity,
    ) -> anyhow::Result<(Vec<NftActivity>, i64)>;

    async fn fetch_nft_listings(
        &self,
        id: &str,
        filter: &FilterListing,
    ) -> anyhow::Result<(Vec<NftListing>, i64)>;
}

pub struct NftService<TDb: IDatabase> {
    db: Arc<TDb>,
}

impl<TDb: IDatabase> NftService<TDb> {
    pub fn new(db: Arc<TDb>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl<TDb> INftService for NftService<TDb>
where
    TDb: IDatabase + Send + Sync + 'static,
{
    async fn fetch_info(&self, id: &str) -> anyhow::Result<NftInfo> {
        self.db.nfts().fetch_nft_info(id).await
    }

    async fn fetch_nft_activities(
        &self,
        id: &str,
        filter: &FilterActivity,
    ) -> anyhow::Result<(Vec<NftActivity>, i64)> {
        let repository = self.db.nfts();

        let filter_fut =
            repository.fetch_nft_activities(id, filter.paging.page, filter.paging.page_size);

        let count_fut = repository.count_nft_activities(&id);

        let (data_res, count_res) = tokio::join!(filter_fut, count_fut);
        let (data, count) = (data_res?, count_res?);

        Ok((data, count))
    }

    async fn fetch_nft_listings(
        &self,
        id: &str,
        filter: &FilterListing,
    ) -> anyhow::Result<(Vec<NftListing>, i64)> {
        let repository = self.db.nfts();

        let filter_fut =
            repository.fetch_nft_listings(id, filter.paging.page, filter.paging.page_size);

        let count_fut = repository.count_nft_listings(&id);

        let (data_res, count_res) = tokio::join!(filter_fut, count_fut);
        let (data, count) = (data_res?, count_res?);

        Ok((data, count))
    }
}
