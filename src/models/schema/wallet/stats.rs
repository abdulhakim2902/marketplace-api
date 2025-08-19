use async_graphql::SimpleObject;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(name = "WalletStats", rename_fields = "snake_case")]
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
