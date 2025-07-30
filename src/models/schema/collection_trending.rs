use crate::models::schema::{
    collection::CollectionSchema, fetch_collection, fetch_nft, nft::NftSchema,
};
use async_graphql::Context;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionTrendingSchema {
    pub nft_id: Option<String>,
    pub collection_id: Option<String>,
    pub tx_frequency: Option<i64>,
    pub last_price: Option<BigDecimal>,
}

#[async_graphql::Object]
impl CollectionTrendingSchema {
    #[graphql(name = "collection_id")]
    async fn collection_id(&self) -> Option<&str> {
        self.collection_id.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "nft_id")]
    async fn nft_id(&self) -> Option<&str> {
        self.nft_id.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "tx_frequency")]
    async fn tx_frequency(&self) -> Option<i64> {
        self.tx_frequency
    }

    #[graphql(name = "last_price")]
    async fn last_price(&self) -> Option<String> {
        self.last_price.as_ref().map(|e| e.to_string())
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        fetch_collection(ctx, self.collection_id.clone()).await
    }

    async fn nft(&self, ctx: &Context<'_>) -> Option<NftSchema> {
        fetch_nft(ctx, self.nft_id.clone(), self.collection_id.clone()).await
    }
}
