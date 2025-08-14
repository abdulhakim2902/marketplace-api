use crate::models::schema::fetch_token_price;
use crate::models::{
    marketplace::APT_DECIMAL,
    schema::{collection::CollectionSchema, fetch_collection, fetch_nft, nft::NftSchema},
};
use async_graphql::{ComplexObject, Context, InputObject, SimpleObject};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
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
    #[graphql(name = "type")]
    pub bid_type: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[ComplexObject]
impl OfferSchema {
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
