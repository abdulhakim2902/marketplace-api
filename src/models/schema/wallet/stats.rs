use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsSchema {
    pub total_buys: Option<i64>,
    pub holding_periods: Option<BigDecimal>,
    pub trade_volumes: Option<BigDecimal>,
    pub unique_nfts: Option<i64>,
    pub total_sales: Option<i64>,
    pub total_mints: Option<i64>,
    pub total_profits: Option<BigDecimal>,
    pub total_usd_profits: Option<BigDecimal>,
}

#[async_graphql::Object]
impl StatsSchema {
    #[graphql(name = "holding_periods")]
    async fn holding_periods(&self) -> Option<String> {
        self.holding_periods.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "trade_volumes")]
    async fn trade_volumes(&self) -> Option<String> {
        self.trade_volumes.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "total_profits")]
    async fn total_profits(&self) -> Option<String> {
        self.total_profits.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "total_usd_profits")]
    async fn total_usd_profits(&self) -> Option<String> {
        self.total_usd_profits.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "total_buys")]
    async fn total_buys(&self) -> Option<i64> {
        self.total_buys
    }

    #[graphql(name = "total_sales")]
    async fn total_sales(&self) -> Option<i64> {
        self.total_sales
    }

    #[graphql(name = "total_mints")]
    async fn total_mints(&self) -> Option<i64> {
        self.total_mints
    }

    #[graphql(name = "unique_nfts")]
    async fn unique_nfts(&self) -> Option<i64> {
        self.unique_nfts
    }
}
