use async_graphql::InputObject;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProfitLeaderboardSchema {
    pub address: Option<String>,
    pub spent: Option<BigDecimal>,
    pub bought: Option<i64>,
    pub sold: Option<i64>,
    pub total_profit: Option<BigDecimal>,
}

#[async_graphql::Object]
impl ProfitLeaderboardSchema {
    async fn address(&self) -> Option<&str> {
        self.address.as_ref().map(|e| e.as_str())
    }

    async fn spent(&self) -> Option<String> {
        self.spent.as_ref().map(|e| e.to_string())
    }

    async fn bought(&self) -> Option<i64> {
        self.bought
    }

    async fn sold(&self) -> Option<i64> {
        self.sold
    }

    #[graphql(name = "total_profit")]
    async fn total_profit(&self) -> Option<String> {
        self.total_profit.as_ref().map(|e| e.to_string())
    }
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
