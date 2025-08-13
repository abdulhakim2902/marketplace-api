use crate::models::schema::fetch_token_price;
use crate::models::{
    marketplace::APT_DECIMAL,
    schema::{collection::CollectionSchema, fetch_collection, fetch_nft, nft::NftSchema},
};
use async_graphql::{Context, InputObject};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OfferSchema {
    pub id: Uuid,
    pub bidder: Option<String>,
    pub accepted_tx_id: Option<String>,
    pub cancelled_tx_id: Option<String>,
    pub created_tx_id: Option<String>,
    pub collection_id: Option<Uuid>,
    pub expired_at: Option<DateTime<Utc>>,
    pub market_contract_id: Option<String>,
    pub market_name: Option<String>,
    pub nonce: Option<String>,
    pub nft_id: Option<Uuid>,
    pub price: Option<i64>,
    pub receiver: Option<String>,
    pub remaining_count: Option<i64>,
    pub status: Option<String>,
    pub bid_type: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[async_graphql::Object]
impl OfferSchema {
    async fn id(&self) -> String {
        self.id.to_string()
    }

    #[graphql(name = "collection_id")]
    async fn collection_id(&self) -> Option<String> {
        self.collection_id.as_ref().map(|e| e.to_string())
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
    async fn nft_id(&self) -> Option<String> {
        self.nft_id.as_ref().map(|e| e.to_string())
    }

    async fn price(&self) -> Option<String> {
        self.price
            .as_ref()
            .map(|e| (BigDecimal::from(*e) / APT_DECIMAL).to_plain_string())
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

    #[graphql(name = "usd_price")]
    async fn usd_price(&self, ctx: &Context<'_>) -> Option<String> {
        let token_price = fetch_token_price(ctx).await.unwrap_or_default();

        self.price
            .map(|e| (BigDecimal::from(e) * token_price / APT_DECIMAL).to_plain_string())
    }

    async fn nft(&self, ctx: &Context<'_>) -> Option<NftSchema> {
        fetch_nft(
            ctx,
            self.nft_id.as_ref().map(|e| e.to_string()),
            self.collection_id.as_ref().map(|e| e.to_string()),
        )
        .await
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        fetch_collection(ctx, self.collection_id.as_ref().map(|e| e.to_string())).await
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterOfferSchema {
    #[graphql(name = "where")]
    pub where_: Option<WhereOfferSchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereOfferSchema {
    pub nft_id: Option<String>,
    pub collection_id: Option<String>,
    pub bidder: Option<String>,
    pub receiver: Option<String>,
    pub status: Option<String>,
    #[graphql(name = "type")]
    pub bid_type: Option<String>,
}
