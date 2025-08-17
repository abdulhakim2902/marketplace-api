use crate::models::schema::nft::QueryNftSchema;
use crate::models::schema::{Date, OperatorSchema, OrderingType, fetch_token_price};
use crate::models::{
    marketplace::APT_DECIMAL,
    schema::{collection::CollectionSchema, fetch_collection, fetch_nft, nft::NftSchema},
};
use async_graphql::{ComplexObject, Context, InputObject, SimpleObject};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject, FromRow)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct BidSchema {
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
impl BidSchema {
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct QueryBidSchema {
    #[graphql(name = "_or")]
    pub _or: Option<Box<QueryBidSchema>>,
    #[graphql(name = "_and")]
    pub _and: Option<Box<QueryBidSchema>>,
    #[graphql(name = "_not")]
    pub _not: Option<Box<QueryBidSchema>>,
    pub id: Option<OperatorSchema<Uuid>>,
    pub bidder: Option<OperatorSchema<String>>,
    pub accepted_tx_id: Option<OperatorSchema<String>>,
    pub cancelled_tx_id: Option<OperatorSchema<String>>,
    pub created_tx_id: Option<OperatorSchema<String>>,
    pub collection_id: Option<OperatorSchema<Uuid>>,
    pub expired_at: Option<OperatorSchema<Date>>,
    pub market_contract_id: Option<OperatorSchema<String>>,
    pub market_name: Option<OperatorSchema<String>>,
    pub nonce: Option<OperatorSchema<String>>,
    pub nft_id: Option<OperatorSchema<Uuid>>,
    pub price: Option<OperatorSchema<i64>>,
    pub receiver: Option<OperatorSchema<String>>,
    pub remaining_count: Option<OperatorSchema<i64>>,
    pub status: Option<OperatorSchema<String>>,
    #[graphql(name = "type")]
    pub bid_type: Option<OperatorSchema<String>>,
    pub nft: Option<QueryNftSchema>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct OrderBidSchema {
    pub id: Option<OrderingType>,
    pub bidder: Option<OrderingType>,
    pub accepted_tx_id: Option<OrderingType>,
    pub cancelled_tx_id: Option<OrderingType>,
    pub created_tx_id: Option<OrderingType>,
    pub collection_id: Option<OrderingType>,
    pub expired_at: Option<OrderingType>,
    pub market_contract_id: Option<OrderingType>,
    pub market_name: Option<OrderingType>,
    pub nonce: Option<OrderingType>,
    pub nft_id: Option<OrderingType>,
    pub price: Option<OrderingType>,
    pub receiver: Option<OrderingType>,
    pub remaining_count: Option<OrderingType>,
    pub status: Option<OrderingType>,
    #[graphql(name = "type")]
    pub bid_type: Option<OrderingType>,
}
