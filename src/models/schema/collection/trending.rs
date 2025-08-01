use crate::models::{
    marketplace::APT_DECIMAL,
    schema::{collection::CollectionSchema, fetch_collection, fetch_nft, nft::NftSchema},
};
use async_graphql::{Context, InputObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TrendingSchema {
    pub nft_id: Option<String>,
    pub collection_id: Option<String>,
    pub tx_frequency: Option<i64>,
    pub last_price: Option<i64>,
}

#[async_graphql::Object]
impl TrendingSchema {
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
        self.last_price
            .as_ref()
            .map(|e| (BigDecimal::from(*e) / APT_DECIMAL).to_plain_string())
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        fetch_collection(ctx, self.collection_id.clone()).await
    }

    async fn nft(&self, ctx: &Context<'_>) -> Option<NftSchema> {
        fetch_nft(ctx, self.nft_id.clone(), self.collection_id.clone()).await
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterTrendingSchema {
    #[graphql(name = "where")]
    pub where_: WhereTrendingSchema,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereTrendingSchema {
    pub collection_id: String,
}
