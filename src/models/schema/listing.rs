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

#[ComplexObject]
impl ListingSchema {
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct QueryListingSchema {
    #[graphql(name = "_or")]
    pub _or: Option<Box<QueryListingSchema>>,
    #[graphql(name = "_and")]
    pub _and: Option<Box<QueryListingSchema>>,
    #[graphql(name = "_not")]
    pub _not: Option<Box<QueryListingSchema>>,
    pub id: Option<OperatorSchema<Uuid>>,
    pub block_height: Option<OperatorSchema<i64>>,
    pub block_time: Option<OperatorSchema<Date>>,
    pub market_contract_id: Option<OperatorSchema<String>>,
    pub listed: Option<OperatorSchema<bool>>,
    pub market_name: Option<OperatorSchema<String>>,
    pub collection_id: Option<OperatorSchema<Uuid>>,
    pub nft_id: Option<OperatorSchema<Uuid>>,
    pub nonce: Option<OperatorSchema<String>>,
    pub price: Option<OperatorSchema<i64>>,
    pub seller: Option<OperatorSchema<String>>,
    pub tx_index: Option<OperatorSchema<i64>>,
    pub nft: Option<QueryNftSchema>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct OrderListingSchema {
    pub id: Option<OrderingType>,
    pub block_height: Option<OrderingType>,
    pub block_time: Option<OrderingType>,
    pub market_contract_id: Option<OrderingType>,
    pub listed: Option<OrderingType>,
    pub market_name: Option<OrderingType>,
    pub collection_id: Option<OrderingType>,
    pub nft_id: Option<OrderingType>,
    pub nonce: Option<OrderingType>,
    pub price: Option<OrderingType>,
    pub seller: Option<OrderingType>,
    pub tx_index: Option<OrderingType>,
}
