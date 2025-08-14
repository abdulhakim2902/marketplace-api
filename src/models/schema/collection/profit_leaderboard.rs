use async_graphql::{InputObject, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct ProfitLeaderboardSchema {
    pub address: Option<String>,
    pub spent: Option<BigDecimal>,
    pub bought: Option<i64>,
    pub sold: Option<i64>,
    pub total_profit: Option<BigDecimal>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterLeaderboardSchema {
    #[graphql(name = "where")]
    pub where_: WhereLeaderboardSchema,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereLeaderboardSchema {
    pub collection_id: String,
}
