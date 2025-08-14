use async_graphql::{ComplexObject, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct CollectionStatSchema {
    pub id: Uuid,
    pub slug: Option<String>,
    pub supply: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub verified: Option<bool>,
    pub website: Option<String>,
    pub discord: Option<String>,
    pub twitter: Option<String>,
    pub royalty: Option<BigDecimal>,
    pub floor: Option<i64>,
    pub owners: Option<i64>,
    pub volume: Option<i64>,
    pub volume_usd: Option<BigDecimal>,
    pub sales: Option<i64>,
    pub listed: Option<i64>,
    pub top_offers: Option<i64>,
    pub volume_24h: Option<i64>,
    pub sales_24h: Option<i64>,
    pub rarity: Option<BigDecimal>,
    pub previous_floor: Option<i64>,
    pub total_offer: Option<i64>,
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
