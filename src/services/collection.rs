use std::sync::Arc;

use crate::{
    database::{IDatabase, collections::ICollections},
    models::api::{
        requests::filter_offer::FilterOffer, responses::collection_offer::CollectionOffer,
    },
};

#[async_trait::async_trait]
pub trait ICollectionService {
    async fn fetch_collection_offers(
        &self,
        id: &str,
        filter: &FilterOffer,
    ) -> anyhow::Result<(Vec<CollectionOffer>, i64)>;
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
    async fn fetch_collection_offers(
        &self,
        id: &str,
        filter: &FilterOffer,
    ) -> anyhow::Result<(Vec<CollectionOffer>, i64)> {
        let repository = self.db.collections();

        let filter_fut =
            repository.fetch_collection_offers(id, filter.paging.page, filter.paging.page_size);

        let count_fut = repository.count_collection_offers(id);

        let (data_res, count_res) = tokio::join!(filter_fut, count_fut);
        let (data, count) = (data_res?, count_res?);

        Ok((data, count))
    }
}
