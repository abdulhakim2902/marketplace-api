use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletStatSchema {
    pub total_buys: Option<i64>,
    pub holding_periods: Option<BigDecimal>,
    pub trade_volumes: Option<BigDecimal>,
    pub unique_nfts: Option<i64>,
}

#[async_graphql::Object]
impl WalletStatSchema {
    #[graphql(name = "total_buys")]
    async fn total_buys(&self) -> Option<i64> {
        self.total_buys
    }

    #[graphql(name = "holding_periods")]
    async fn holding_periods(&self) -> Option<String> {
        self.holding_periods.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "trade_volumes")]
    async fn trade_volumes(&self) -> Option<String> {
        self.trade_volumes.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "unique_nfts")]
    async fn unique_nfts(&self) -> Option<i64> {
        self.unique_nfts
    }
}
