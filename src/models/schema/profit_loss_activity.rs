use async_graphql::{Context, InputObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::models::schema::{
    collection::CollectionSchema, fetch_collection, fetch_nft, nft::NftSchema,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProfitLossActivitySchema {
    pub collection_id: Option<String>,
    pub nft_id: Option<String>,
    pub bought: Option<BigDecimal>,
    pub sold: Option<BigDecimal>,
    pub bought_usd: Option<BigDecimal>,
    pub sold_usd: Option<BigDecimal>,
}

#[async_graphql::Object]
impl ProfitLossActivitySchema {
    #[graphql(name = "collection_id")]
    async fn collection_id(&self) -> Option<&str> {
        self.collection_id.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "nft_id")]
    async fn nft_id(&self) -> Option<&str> {
        self.nft_id.as_ref().map(|e| e.as_str())
    }

    async fn bought(&self) -> Option<String> {
        self.bought.as_ref().map(|e| e.to_string())
    }

    async fn sold(&self) -> Option<String> {
        self.sold.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "bought_usd")]
    async fn bought_usd(&self) -> Option<String> {
        self.bought_usd.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "sold_usd")]
    async fn sold_usd(&self) -> Option<String> {
        self.sold_usd.as_ref().map(|e| e.to_string())
    }

    async fn nft(&self, ctx: &Context<'_>) -> Option<NftSchema> {
        fetch_nft(ctx, self.nft_id.clone(), self.collection_id.clone()).await
    }

    async fn collecton(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        fetch_collection(ctx, self.collection_id.clone()).await
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterProfitLossActivitiesSchema {
    #[graphql(name = "where")]
    pub where_: Option<WalletProfitLossWhereSchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WalletProfitLossWhereSchema {
    pub wallet_address: Option<String>,
}
