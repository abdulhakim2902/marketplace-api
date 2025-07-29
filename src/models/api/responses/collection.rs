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
    async fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|e| e.as_str())
    }

    async fn slug(&self) -> Option<&str> {
        self.slug.as_ref().map(|e| e.as_str())
    }

    async fn supply(&self) -> Option<i64> {
        self.supply
    }

    async fn title(&self) -> Option<&str> {
        self.title.as_ref().map(|e| e.as_str())
    }

    async fn description(&self) -> Option<&str> {
        self.description.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "cover_url")]
    async fn cover_url(&self) -> Option<&str> {
        self.cover_url.as_ref().map(|e| e.as_str())
    }

    async fn floor(&self) -> Option<String> {
        self.floor.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "prev_floor")]
    async fn prev_floor(&self) -> Option<String> {
        self.prev_floor.as_ref().map(|e| e.to_plain_string())
    }

    async fn owners(&self) -> Option<i64> {
        self.owners
    }

    async fn sales(&self) -> Option<i64> {
        self.sales
    }

    async fn listed(&self) -> Option<i64> {
        self.listed
    }

    #[graphql(name = "top_offer")]
    async fn top_offer(&self) -> Option<String> {
        self.top_offer.as_ref().map(|e| e.to_plain_string())
    }

    async fn volume(&self) -> Option<String> {
        self.volume.as_ref().map(|e| e.to_plain_string())
    }

    async fn average(&self) -> Option<String> {
        self.average.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "volume_usd")]
    async fn volume_usd(&self) -> Option<String> {
        self.volume_usd.as_ref().map(|e| e.to_plain_string())
    }
}
