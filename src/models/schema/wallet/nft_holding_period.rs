use crate::models::schema::{
    collection::CollectionSchema, fetch_collection, fetch_nft, nft::NftSchema,
};
use async_graphql::Context;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NftHoldingPeriod {
    pub collection_id: Option<String>,
    pub nft_id: Option<String>,
    pub period: Option<BigDecimal>,
}

#[async_graphql::Object]
impl NftHoldingPeriod {
    #[graphql(name = "collection_id")]
    async fn collection_id(&self) -> Option<String> {
        self.collection_id.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "nft_id")]
    async fn nft_id(&self) -> Option<String> {
        self.nft_id.as_ref().map(|e| e.to_string())
    }

    async fn period(&self) -> Option<String> {
        self.period.as_ref().map(|e| e.to_plain_string())
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        fetch_collection(ctx, self.collection_id.clone()).await
    }

    async fn nft(&self, ctx: &Context<'_>) -> Option<NftSchema> {
        fetch_nft(ctx, self.nft_id.clone(), self.collection_id.clone()).await
    }
}
