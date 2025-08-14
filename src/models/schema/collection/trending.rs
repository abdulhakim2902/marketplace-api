use crate::models::schema::{
    collection::CollectionSchema, fetch_collection, fetch_nft, nft::NftSchema,
};
use async_graphql::{ComplexObject, Context, InputObject, SimpleObject};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct TrendingSchema {
    pub nft_id: Uuid,
    pub collection_id: Option<Uuid>,
    pub tx_frequency: Option<i64>,
    pub last_price: Option<i64>,
}

#[ComplexObject]
impl TrendingSchema {
    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        fetch_collection(ctx, self.collection_id.as_ref().map(|e| e.to_string())).await
    }

    async fn nft(&self, ctx: &Context<'_>) -> Option<NftSchema> {
        fetch_nft(
            ctx,
            Some(self.nft_id.to_string()),
            self.collection_id.as_ref().map(|e| e.to_string()),
        )
        .await
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
