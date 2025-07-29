use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Clone, Debug, Deserialize, Serialize, FromRow)]
pub struct CollectionNft {
    pub id: Option<String>,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub listing_price: Option<BigDecimal>,
    pub listing_usd_price: Option<BigDecimal>,
    pub last_sale: Option<BigDecimal>,
    pub listed_at: Option<DateTime<Utc>>,
    pub top_offer: Option<BigDecimal>,
    pub royalty: Option<BigDecimal>,
    pub rarity_score: Option<BigDecimal>,
}

#[async_graphql::Object]
impl CollectionNft {
    async fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|e| e.as_str())
    }

    async fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|e| e.as_str())
    }

    async fn owner(&self) -> Option<&str> {
        self.owner.as_ref().map(|e| e.as_str())
    }

    async fn description(&self) -> Option<&str> {
        self.description.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "image_url")]
    async fn image_url(&self) -> Option<&str> {
        self.image_url.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "listing_price")]
    async fn listing_price(&self) -> Option<String> {
        self.listing_price.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "listing_usd_price")]
    async fn listing_usd_price(&self) -> Option<String> {
        self.listing_usd_price.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "last_sale")]
    async fn last_sale(&self) -> Option<String> {
        self.last_sale.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "listed_at")]
    async fn listed_at(&self) -> Option<String> {
        self.listed_at.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "top_offer")]
    async fn top_offer(&self) -> Option<String> {
        self.top_offer.as_ref().map(|e| e.to_plain_string())
    }

    async fn royalty(&self) -> Option<String> {
        self.royalty.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "rarity_score")]
    async fn rarity_score(&self) -> Option<String> {
        self.rarity_score.as_ref().map(|e| e.to_plain_string())
    }
}
