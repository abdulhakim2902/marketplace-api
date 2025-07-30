use async_graphql::InputObject;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BidSchema {
    pub id: Option<String>,
    pub bidder: Option<String>,
    pub accepted_tx_id: Option<String>,
    pub cancelled_tx_id: Option<String>,
    pub created_tx_id: Option<String>,
    pub collection_id: Option<String>,
    pub expired_at: Option<DateTime<Utc>>,
    pub market_contract_id: Option<String>,
    pub market_name: Option<String>,
    pub nonce: Option<String>,
    pub nft_id: Option<String>,
    pub price: Option<BigDecimal>,
    pub receiver: Option<String>,
    pub remaining_count: Option<i64>,
    pub status: Option<String>,
    pub bid_type: Option<String>,
    pub price_str: Option<String>,
    pub usd_price: Option<BigDecimal>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[async_graphql::Object]
impl BidSchema {
    #[graphql(name = "collection_id")]
    async fn collection_id(&self) -> Option<&str> {
        self.collection_id.as_ref().map(|e| e.as_str())
    }

    async fn bidder(&self) -> Option<&str> {
        self.bidder.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "accepted_tx_id")]
    async fn accepted_tx_id(&self) -> Option<&str> {
        self.accepted_tx_id.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "cancelled_tx_id")]
    async fn cancelled_tx_id(&self) -> Option<&str> {
        self.cancelled_tx_id.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "created_tx_id")]
    async fn created_tx_id(&self) -> Option<&str> {
        self.created_tx_id.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "expired_at")]
    async fn expired_at(&self) -> Option<String> {
        self.expired_at.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "updated_at")]
    async fn updated_at(&self) -> Option<String> {
        self.updated_at.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "market_contract_id")]
    async fn market_contract_id(&self) -> Option<&str> {
        self.market_contract_id.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "market_name")]
    async fn market_name(&self) -> Option<&str> {
        self.market_name.as_ref().map(|e| e.as_str())
    }

    async fn nonce(&self) -> Option<&str> {
        self.nonce.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "nft_id")]
    async fn nft_id(&self) -> Option<&str> {
        self.nft_id.as_ref().map(|e| e.as_str())
    }

    async fn price(&self) -> Option<String> {
        self.price.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "usd_price")]
    async fn usd_price(&self) -> Option<String> {
        self.usd_price.as_ref().map(|e| e.to_string())
    }

    async fn receiver(&self) -> Option<&str> {
        self.receiver.as_ref().map(|e| e.as_str())
    }

    async fn amount(&self) -> Option<i64> {
        self.remaining_count
    }

    async fn status(&self) -> Option<&str> {
        self.status.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "type")]
    async fn bid_type(&self) -> Option<&str> {
        self.bid_type.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "total_collection_trait")]
    async fn total_collection_trait(&self) -> i64 {
        10
    }

    #[graphql(name = "total_nft_trait")]
    async fn total_nft_trait(&self) -> i64 {
        5
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterBidSchema {
    #[graphql(name = "where")]
    pub where_: Option<WhereSchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereSchema {
    pub nft_id: Option<String>,
    pub collection_id: Option<String>,
}
