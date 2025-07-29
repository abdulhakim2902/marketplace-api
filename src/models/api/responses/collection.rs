use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow)]
pub struct Collection {
    pub id: Option<String>,
    pub slug: Option<String>,
    pub supply: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub floor: Option<BigDecimal>,
    pub prev_floor: Option<BigDecimal>,
    pub owners: Option<i64>,
    pub sales: Option<i64>,
    pub listed: Option<i64>,
    pub top_offer: Option<BigDecimal>,
    pub volume: Option<BigDecimal>,
    pub average: Option<BigDecimal>,
    pub volume_usd: Option<BigDecimal>,
}

#[async_graphql::Object]
impl Collection {
    async fn id(&self) -> Option<&String> {
        self.id.as_ref()
    }
    async fn slug(&self) -> Option<&String> {
        self.slug.as_ref()
    }
}
