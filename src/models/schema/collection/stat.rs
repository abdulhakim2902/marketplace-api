use crate::models::marketplace::APT_DECIMAL;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow)]
pub struct CollectionStatSchema {
    pub id: Option<String>,
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

#[async_graphql::Object]
impl CollectionStatSchema {
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

    async fn verified(&self) -> Option<bool> {
        self.verified
    }

    async fn website(&self) -> Option<&str> {
        self.website.as_ref().map(|e| e.as_str())
    }

    async fn discord(&self) -> Option<&str> {
        self.discord.as_ref().map(|e| e.as_str())
    }

    async fn twitter(&self) -> Option<&str> {
        self.twitter.as_ref().map(|e| e.as_str())
    }

    async fn royalty(&self) -> Option<String> {
        self.royalty.as_ref().map(|e| e.to_plain_string())
    }

    async fn floor(&self) -> Option<String> {
        self.floor
            .as_ref()
            .map(|e| (BigDecimal::from(*e) / APT_DECIMAL).to_plain_string())
    }

    async fn owners(&self) -> Option<i64> {
        self.owners
    }

    async fn volume(&self) -> Option<String> {
        self.volume
            .as_ref()
            .map(|e| (BigDecimal::from(*e) / APT_DECIMAL).to_plain_string())
    }

    #[graphql(name = "volume_usd")]
    async fn volume_usd(&self) -> Option<String> {
        self.volume_usd.as_ref().map(|e| e.to_plain_string())
    }

    async fn sales(&self) -> Option<i64> {
        self.sales
    }

    async fn listed(&self) -> Option<i64> {
        self.listed
    }

    #[graphql(name = "market_cap")]
    async fn market_cap(&self) -> Option<String> {
        self.supply
            .zip(self.floor)
            .map(|(supply, floor)| BigDecimal::from(supply * floor) / APT_DECIMAL)
            .map(|value| value.to_plain_string())
    }

    #[graphql(name = "top_offers")]
    async fn top_offers(&self) -> Option<String> {
        self.top_offers
            .as_ref()
            .map(|e| (BigDecimal::from(*e) / APT_DECIMAL).to_plain_string())
    }

    #[graphql(name = "volume_24h")]
    async fn volume_24h(&self) -> Option<String> {
        self.volume_24h
            .as_ref()
            .map(|e| (BigDecimal::from(*e) / APT_DECIMAL).to_plain_string())
    }

    #[graphql(name = "sales_24h")]
    async fn sales_24h(&self) -> Option<i64> {
        self.sales_24h
    }

    async fn rarity(&self) -> Option<String> {
        self.rarity.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "previous_floor")]
    async fn previous_floor(&self) -> Option<String> {
        self.previous_floor
            .as_ref()
            .map(|e| (BigDecimal::from(*e) / APT_DECIMAL).to_plain_string())
    }

    #[graphql(name = "total_offer")]
    async fn total_offer(&self) -> Option<String> {
        self.total_offer
            .as_ref()
            .map(|e| (BigDecimal::from(*e) / APT_DECIMAL).to_plain_string())
    }
}
