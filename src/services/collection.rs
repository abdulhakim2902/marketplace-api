use std::sync::Arc;

use crate::{
    database::{IDatabase, collections::ICollections},
    models::api::{
        requests::{
            filter_activity::FilterActivity, filter_collection::FilterCollection,
            filter_nft::FilterNft, filter_nft_change::FilterNftChange,
            filter_nft_holder::FilterNftHolder, filter_nft_trending::FilterNftTrending,
            filter_offer::FilterOffer, filter_profit_leaderboard::FilterProfitLeader,
            filter_top_buyer::FilterTopBuyer, filter_top_seller::FilterTopSeller,
            floor_chart::FloorChart,
        },
        responses::{
            collection::Collection,
            collection_activity::CollectionActivity,
            collection_info::CollectionInfo,
            collection_nft::CollectionNft,
            collection_nft_change::CollectionNftChange,
            collection_nft_distribution::{
                CollectionNftAmountDistribution, CollectionNftPeriodDistribution,
            },
            collection_nft_holder::CollectionNftHolder,
            collection_nft_trending::CollectionNftTrending,
            collection_offer::CollectionOffer,
            collection_profit_leaderboard::CollectionProfitLeaderboard,
            collection_top_buyer::CollectionTopBuyer,
            collection_top_seller::CollectionTopSeller,
            data_point::DataPoint,
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

    async fn fetch_collection_offers(
        &self,
        id: &str,
        filter: &FilterOffer,
    ) -> anyhow::Result<(Vec<CollectionOffer>, i64)>;

    async fn fetch_collection_activities(
        &self,
        id: &str,
        filter: &FilterActivity,
    ) -> anyhow::Result<(Vec<CollectionActivity>, i64)>;

    async fn fetch_collection_floor_chart(
        &self,
        id: &str,
        floor_chart: &FloorChart,
    ) -> anyhow::Result<Vec<DataPoint>>;

    async fn fetch_collection_top_buyer(
        &self,
        id: &str,
        filter: &FilterTopBuyer,
    ) -> anyhow::Result<Vec<CollectionTopBuyer>>;

    async fn fetch_collection_top_seller(
        &self,
        id: &str,
        filter: &FilterTopSeller,
    ) -> anyhow::Result<Vec<CollectionTopSeller>>;

    async fn fetch_collection_nft_holders(
        &self,
        id: &str,
        filter: &FilterNftHolder,
    ) -> anyhow::Result<(Vec<CollectionNftHolder>, i64)>;

    async fn fetch_collection_trending_nfts(
        &self,
        id: &str,
        filter: &FilterNftTrending,
    ) -> anyhow::Result<(Vec<CollectionNftTrending>, i64)>;

    async fn fetch_collection_nft_amount_distribution(
        &self,
        id: &str,
    ) -> anyhow::Result<CollectionNftAmountDistribution>;

    async fn fetch_collection_nft_period_distribution(
        &self,
        id: &str,
    ) -> anyhow::Result<CollectionNftPeriodDistribution>;

    async fn fetch_collection_profit_leaderboard(
        &self,
        id: &str,
        filter: &FilterProfitLeader,
    ) -> anyhow::Result<(Vec<CollectionProfitLeaderboard>, i64)>;

    async fn fetch_collection_nft_change(
        &self,
        id: &str,
        filter: &FilterNftChange,
    ) -> anyhow::Result<(Vec<CollectionNftChange>, i64)>;
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

        let filter_fut = repository.filter(filter.interval, filter.limit, filter.offset);
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

        let filter_fut = repository.fetch_collection_nfts(id, filter.limit, filter.offset);

        let count_fut = repository.count_collection_nfts(id);

        let (data_res, count_res) = tokio::join!(filter_fut, count_fut);
        let (data, count) = (data_res?, count_res?);

        Ok((data, count))
    }

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

    async fn fetch_collection_activities(
        &self,
        id: &str,
        filter: &FilterActivity,
    ) -> anyhow::Result<(Vec<CollectionActivity>, i64)> {
        let repository = self.db.collections();

        let filter_fut =
            repository.fetch_collection_activities(id, filter.paging.page, filter.paging.page_size);

        let count_fut = repository.count_collection_activities(id);

        let (data_res, count_res) = tokio::join!(filter_fut, count_fut);
        let (data, count) = (data_res?, count_res?);

        Ok((data, count))
    }

    async fn fetch_collection_floor_chart(
        &self,
        id: &str,
        floor_chart: &FloorChart,
    ) -> anyhow::Result<Vec<DataPoint>> {
        self.db
            .collections()
            .fetch_collection_floor_chart(
                id,
                floor_chart.time_range.start_time,
                floor_chart.time_range.end_time,
                floor_chart.time_range.interval,
            )
            .await
    }

    async fn fetch_collection_top_buyer(
        &self,
        id: &str,
        filter: &FilterTopBuyer,
    ) -> anyhow::Result<Vec<CollectionTopBuyer>> {
        self.db
            .collections()
            .fetch_collection_top_buyers(id, filter.interval)
            .await
    }

    async fn fetch_collection_top_seller(
        &self,
        id: &str,
        filter: &FilterTopSeller,
    ) -> anyhow::Result<Vec<CollectionTopSeller>> {
        self.db
            .collections()
            .fetch_collection_top_sellers(id, filter.interval)
            .await
    }

    async fn fetch_collection_nft_holders(
        &self,
        id: &str,
        filter: &FilterNftHolder,
    ) -> anyhow::Result<(Vec<CollectionNftHolder>, i64)> {
        let repository = self.db.collections();

        let filter_fut = repository.fetch_collection_nft_holders(
            id,
            filter.paging.page,
            filter.paging.page_size,
        );

        let count_fut = repository.count_collection_nft_holders(id);

        let (data_res, count_res) = tokio::join!(filter_fut, count_fut);
        let (data, count) = (data_res?, count_res?);

        Ok((data, count))
    }

    async fn fetch_collection_trending_nfts(
        &self,
        id: &str,
        filter: &FilterNftTrending,
    ) -> anyhow::Result<(Vec<CollectionNftTrending>, i64)> {
        let repository = self.db.collections();

        let filter_fut = repository.fetch_collection_trending_nfts(
            id,
            filter.paging.page,
            filter.paging.page_size,
        );

        let count_fut = repository.count_collection_trending_nfts(id);

        let (data_res, count_res) = tokio::join!(filter_fut, count_fut);
        let (data, count) = (data_res?, count_res?);

        Ok((data, count))
    }

    async fn fetch_collection_nft_amount_distribution(
        &self,
        id: &str,
    ) -> anyhow::Result<CollectionNftAmountDistribution> {
        self.db
            .collections()
            .fetch_collection_nft_amount_distribution(id)
            .await
    }

    async fn fetch_collection_nft_period_distribution(
        &self,
        id: &str,
    ) -> anyhow::Result<CollectionNftPeriodDistribution> {
        self.db
            .collections()
            .fetch_collection_nft_period_distribution(id)
            .await
    }

    async fn fetch_collection_profit_leaderboard(
        &self,
        id: &str,
        filter: &FilterProfitLeader,
    ) -> anyhow::Result<(Vec<CollectionProfitLeaderboard>, i64)> {
        let repository = self.db.collections();

        let filter_fut = repository.fetch_collection_profit_leaderboard(
            id,
            filter.paging.page,
            filter.paging.page_size,
        );

        let count_fut = repository.count_collection_profit_leaderboard(id);

        let (data_res, count_res) = tokio::join!(filter_fut, count_fut);
        let (data, count) = (data_res?, count_res?);

        Ok((data, count))
    }

    async fn fetch_collection_nft_change(
        &self,
        id: &str,
        filter: &FilterNftChange,
    ) -> anyhow::Result<(Vec<CollectionNftChange>, i64)> {
        let repository = self.db.collections();

        let filter_fut = repository.fetch_collection_nft_change(
            id,
            filter.interval,
            filter.paging.page,
            filter.paging.page_size,
        );

        let count_fut = repository.count_collection_nft_change(id, filter.interval);

        let (data_res, count_res) = tokio::join!(filter_fut, count_fut);
        let (data, count) = (data_res?, count_res?);

        Ok((data, count))
    }
}
