use async_graphql::{ComplexObject, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(complex, name = "CollectionStats", rename_fields = "snake_case")]
pub struct CollectionStatSchema {
    pub floor: Option<i64>,
    pub owners: Option<i64>,
    pub listed: Option<i64>,
    pub supply: Option<i64>,
    pub total_volume: Option<i64>,
    pub total_usd_volume: Option<BigDecimal>,
    pub total_sales: Option<i64>,
    pub day_volume: Option<i64>,
    pub day_sales: Option<i64>,
    pub top_offer: Option<i64>,
    pub rarity: Option<BigDecimal>,
}

#[ComplexObject]
impl CollectionStatSchema {
    #[graphql(name = "market_cap")]
    async fn market_cap(&self) -> Option<i64> {
        self.supply
            .zip(self.floor)
            .map(|(supply, floor)| supply * floor)
    }
}
