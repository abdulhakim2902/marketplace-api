use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(name = "CollectionNftAmountDistribution", rename_fields = "snake_case")]
pub struct NftAmountDistributionSchema {
    pub range_1: Option<i64>,
    pub range_2_to_3: Option<i64>,
    pub range_4_to_10: Option<i64>,
    pub range_11_to_50: Option<i64>,
    pub range_51_to_100: Option<i64>,
    pub range_gt_100: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(name = "CollectionNftPeriodDistribution", rename_fields = "snake_case")]
pub struct NftPeriodDistributionSchema {
    pub range_lt_24h: Option<i64>,
    pub range_1d_to_7d: Option<i64>,
    pub range_7d_to_30d: Option<i64>,
    pub range_30d_to_3m: Option<i64>,
    pub range_3m_to_1y: Option<i64>,
    pub range_gte_1y: Option<i64>,
}
