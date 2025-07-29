use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow)]
pub struct CollectionActivity {
    pub tx_type: Option<String>,
    pub tx_index: Option<i64>,
    pub tx_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub price: Option<BigDecimal>,
    pub usd_price: Option<BigDecimal>,
    pub market_name: Option<String>,
    pub market_contract_id: Option<String>,
    pub time: Option<DateTime<Utc>>,
    pub nft_id: Option<String>,
    pub nft_name: Option<String>,
    pub nft_description: Option<String>,
    pub nft_image_url: Option<String>,
    pub quantity: Option<i64>,
    pub rarity_score: Option<BigDecimal>,
}

#[async_graphql::Object]
impl CollectionActivity {
    #[graphql(name = "tx_type")]
    async fn tx_type(&self) -> Option<&str> {
        self.tx_type.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "tx_index")]
    async fn tx_index(&self) -> Option<i64> {
        self.tx_index
    }

    #[graphql(name = "tx_id")]
    async fn tx_id(&self) -> Option<&str> {
        self.tx_id.as_ref().map(|e| e.as_str())
    }

    async fn from(&self) -> Option<&str> {
        self.from.as_ref().map(|e| e.as_str())
    }

    async fn to(&self) -> Option<&str> {
        self.to.as_ref().map(|e| e.as_str())
    }

    async fn price(&self) -> Option<String> {
        self.price.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "usd_price")]
    async fn usd_price(&self) -> Option<String> {
        self.usd_price.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "market_name")]
    async fn market_name(&self) -> Option<&str> {
        self.market_name.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "market_contract_id")]
    async fn market_contract_id(&self) -> Option<&str> {
        self.market_contract_id.as_ref().map(|e| e.as_str())
    }

    async fn time(&self) -> Option<String> {
        self.time.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "nft_id")]
    async fn nft_id(&self) -> Option<&str> {
        self.nft_id.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "nft_name")]
    async fn nft_name(&self) -> Option<&str> {
        self.nft_name.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "nft_description")]
    async fn nft_description(&self) -> Option<&str> {
        self.nft_description.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "nft_image_url")]
    async fn nft_image_url(&self) -> Option<&str> {
        self.nft_image_url.as_ref().map(|e| e.as_str())
    }

    async fn quantity(&self) -> Option<i64> {
        self.quantity
    }

    #[graphql(name = "rarity_score")]
    async fn rarity_score(&self) -> Option<String> {
        self.rarity_score.as_ref().map(|e| e.to_plain_string())
    }
}
