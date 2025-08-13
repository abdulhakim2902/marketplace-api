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
pub struct ListingSchema {
    pub id: Uuid,
    pub block_height: Option<i64>,
    pub block_time: Option<DateTime<Utc>>,
    pub market_contract_id: Option<String>,
    pub listed: Option<bool>,
    pub market_name: Option<String>,
    pub collection_id: Option<Uuid>,
    pub nft_id: Option<Uuid>,
    pub nonce: Option<String>,
    pub price: Option<i64>,
    pub seller: Option<String>,
    pub tx_index: Option<i64>,
}

#[async_graphql::Object]
impl ListingSchema {
    async fn id(&self) -> String {
        self.id.to_string()
    }

    #[graphql(name = "block_height")]
    async fn block_height(&self) -> Option<i64> {
        self.block_height
    }

    #[graphql(name = "block_time")]
    async fn block_time(&self) -> Option<String> {
        self.block_time.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "market_contract_id")]
    async fn market_contract_id(&self) -> Option<&str> {
        self.market_contract_id.as_ref().map(|e| e.as_str())
    }

    async fn listed(&self) -> Option<bool> {
        self.listed
    }

    #[graphql(name = "market_name")]
    async fn market_name(&self) -> Option<&str> {
        self.market_name.as_ref().map(|e| e.as_str())
    }

    async fn nonce(&self) -> Option<&str> {
        self.nonce.as_ref().map(|e| e.as_str())
    }

    async fn price(&self) -> Option<String> {
        self.price
            .map(|e| (BigDecimal::from(e) / APT_DECIMAL).to_plain_string())
    }

    async fn seller(&self) -> Option<&str> {
        self.seller.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "tx_index")]
    async fn tx_index(&self) -> Option<i64> {
        self.tx_index
    }

    #[graphql(name = "collection_id")]
    async fn collection_id(&self) -> Option<String> {
        self.collection_id.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "nft_id")]
    async fn nft_id(&self) -> Option<String> {
        self.nft_id.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "usd_price")]
    async fn usd_price(&self, ctx: &Context<'_>) -> Option<String> {
        let token_price = fetch_token_price(ctx).await.unwrap_or_default();

        self.price
            .map(|e| (BigDecimal::from(e) * token_price / APT_DECIMAL).to_plain_string())
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        fetch_collection(ctx, self.collection_id.as_ref().map(|e| e.to_string())).await
    }

    async fn nft(&self, ctx: &Context<'_>) -> Option<NftSchema> {
        fetch_nft(
            ctx,
            self.nft_id.as_ref().map(|e| e.to_string()),
            self.collection_id.as_ref().map(|e| e.to_string()),
        )
        .await
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterListingSchema {
    #[graphql(name = "where")]
    pub where_: Option<WhereListingSchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereListingSchema {
    pub id: Option<String>,
    pub nft_id: Option<String>,
    pub is_listed: Option<bool>,
}
