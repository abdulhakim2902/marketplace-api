use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NftAmountDistributionSchema {
    pub range_1: Option<i64>,
    pub range_2_to_3: Option<i64>,
    pub range_4_to_10: Option<i64>,
    pub range_11_to_50: Option<i64>,
    pub range_51_to_100: Option<i64>,
    pub range_gt_100: Option<i64>,
}

#[async_graphql::Object]
impl NftAmountDistributionSchema {
    #[graphql(name = "range_1")]
    async fn range_1(&self) -> Option<i64> {
        self.range_1
    }

    #[graphql(name = "range_2_to_3")]
    async fn range_2_to_3(&self) -> Option<i64> {
        self.range_2_to_3
    }

    #[graphql(name = "range_4_to_10")]
    async fn range_4_to_10(&self) -> Option<i64> {
        self.range_4_to_10
    }

    #[graphql(name = "range_11_to_50")]
    async fn range_11_to_50(&self) -> Option<i64> {
        self.range_11_to_50
    }

    #[graphql(name = "range_51_to_100")]
    async fn range_51_to_100(&self) -> Option<i64> {
        self.range_51_to_100
    }

    #[graphql(name = "range_gt_100")]
    async fn range_gt_100(&self) -> Option<i64> {
        self.range_gt_100
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NftPeriodDistributionSchema {
    pub range_lt_24h: Option<i64>,
    pub range_1d_to_7d: Option<i64>,
    pub range_7d_to_30d: Option<i64>,
    pub range_30d_to_3m: Option<i64>,
    pub range_3m_to_1y: Option<i64>,
    pub range_gte_1y: Option<i64>,
}

#[async_graphql::Object]
impl NftPeriodDistributionSchema {
    #[graphql(name = "range_lt_24h")]
    async fn range_lt_24h(&self) -> Option<i64> {
        self.range_lt_24h
    }

    #[graphql(name = "range_1d_to_7d")]
    async fn range_1d_to_7d(&self) -> Option<i64> {
        self.range_1d_to_7d
    }

    #[graphql(name = "range_7d_to_30d")]
    async fn range_7d_to_30d(&self) -> Option<i64> {
        self.range_7d_to_30d
    }

    #[graphql(name = "range_30d_to_3m")]
    async fn range_30d_to_3m(&self) -> Option<i64> {
        self.range_30d_to_3m
    }

    #[graphql(name = "range_3m_to_1y")]
    async fn range_3m_to_1y(&self) -> Option<i64> {
        self.range_3m_to_1y
    }

    #[graphql(name = "range_gte_1y")]
    async fn range_gte_1y(&self) -> Option<i64> {
        self.range_gte_1y
    }
}
