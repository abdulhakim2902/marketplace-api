use crate::models::{
    marketplace::APT_DECIMAL,
    schema::{collection::CollectionSchema, fetch_collection, fetch_nft, nft::NftSchema},
};
use async_graphql::{Context, InputObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProfitLossSchema {
    pub collection_id: Option<Uuid>,
    pub nft_id: Option<Uuid>,
    pub bought: Option<i64>,
    pub sold: Option<i64>,
    pub bought_usd: Option<BigDecimal>,
    pub sold_usd: Option<BigDecimal>,
}

#[async_graphql::Object]
impl ProfitLossSchema {
    #[graphql(name = "collection_id")]
    async fn collection_id(&self) -> Option<String> {
        self.collection_id.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "nft_id")]
    async fn nft_id(&self) -> Option<String> {
        self.nft_id.as_ref().map(|e| e.to_string())
    }

    async fn bought(&self) -> Option<String> {
        self.bought
            .as_ref()
            .map(|e| (BigDecimal::from(*e) / APT_DECIMAL).to_plain_string())
    }

    async fn sold(&self) -> Option<String> {
        self.sold
            .as_ref()
            .map(|e| (BigDecimal::from(*e) / APT_DECIMAL).to_plain_string())
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
        fetch_nft(
            ctx,
            self.nft_id.as_ref().map(|e| e.to_string()),
            self.collection_id.as_ref().map(|e| e.to_string()),
        )
        .await
    }

    async fn collecton(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        fetch_collection(ctx, self.collection_id.as_ref().map(|e| e.to_string())).await
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterProfitLossSchema {
    #[graphql(name = "where")]
    pub where_: Option<WhereProfitLossSchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereProfitLossSchema {
    pub wallet_address: Option<String>,
}
