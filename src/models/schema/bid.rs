use std::sync::Arc;

use crate::{
    database::{Database, IDatabase, collections::Collections, nfts::Nfts},
    models::{
        marketplace::APT_DECIMAL,
        schema::{
            Date, OperatorSchema, OrderingType,
            collection::{CollectionSchema, OrderCollectionSchema, QueryCollectionSchema},
            fetch_token_price,
            nft::{NftSchema, OrderNftSchema, QueryNftSchema},
        },
    },
};
use async_graphql::{ComplexObject, Context, InputObject, SimpleObject, dataloader::DataLoader};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject, FromRow)]
#[graphql(complex, name = "Bid", rename_fields = "snake_case")]
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
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let data_loader = DataLoader::new(Nfts::new(Arc::new(db.get_pool().clone())), tokio::spawn);

        if let Some(nft_id) = self.nft_id.as_ref() {
            data_loader.load_one(nft_id.clone()).await.ok().flatten()
        } else {
            None
        }
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let data_loader = DataLoader::new(
            Collections::new(Arc::new(db.get_pool().clone())),
            tokio::spawn,
        );

        if let Some(collection_id) = self.collection_id.as_ref() {
            data_loader
                .load_one(collection_id.clone())
                .await
                .ok()
                .flatten()
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(name = "BidQuery", rename_fields = "snake_case")]
pub struct QueryBidSchema {
    #[graphql(name = "_or")]
    pub _or: Option<Arc<QueryBidSchema>>,
    #[graphql(name = "_and")]
    pub _and: Option<Arc<QueryBidSchema>>,
    #[graphql(name = "_not")]
    pub _not: Option<Arc<QueryBidSchema>>,
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
    pub collection: Option<Arc<QueryCollectionSchema>>,
    pub nft: Option<QueryNftSchema>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(name = "BidOrderBy", rename_fields = "snake_case")]
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
    pub collection: Option<OrderCollectionSchema>,
    pub nft: Option<OrderNftSchema>,
}
