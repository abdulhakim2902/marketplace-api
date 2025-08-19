pub mod profit_loss;

use std::sync::Arc;

use crate::{
    database::{Database, IDatabase, collections::Collections, nfts::Nfts},
    models::schema::{
        Date, OperatorSchema, OrderingType,
        collection::{CollectionSchema, OrderCollectionSchema, QueryCollectionSchema},
        nft::{NftSchema, OrderNftSchema, QueryNftSchema},
    },
};
use async_graphql::{ComplexObject, Context, InputObject, SimpleObject, dataloader::DataLoader};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(complex, name = "Activity", rename_fields = "snake_case")]
pub struct ActivitySchema {
    pub id: Uuid,
    #[graphql(name = "type")]
    pub tx_type: Option<String>,
    pub tx_index: i64,
    pub tx_id: String,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub price: Option<i64>,
    pub usd_price: Option<BigDecimal>,
    pub market_name: Option<String>,
    pub market_contract_id: Option<String>,
    pub nft_id: Option<Uuid>,
    pub collection_id: Option<Uuid>,
    pub block_time: Option<DateTime<Utc>>,
    pub block_height: Option<i64>,
    pub amount: Option<i64>,
}

#[ComplexObject]
impl ActivitySchema {
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
#[graphql(name = "ActivityQuery", rename_fields = "snake_case")]
pub struct QueryActivitySchema {
    #[graphql(name = "_or")]
    pub _or: Option<Arc<QueryActivitySchema>>,
    #[graphql(name = "_and")]
    pub _and: Option<Arc<QueryActivitySchema>>,
    #[graphql(name = "_not")]
    pub _not: Option<Arc<QueryActivitySchema>>,
    pub id: Option<OperatorSchema<Uuid>>,
    #[graphql(name = "type")]
    pub tx_type: Option<OperatorSchema<String>>,
    pub tx_index: Option<OperatorSchema<i64>>,
    pub tx_id: Option<OperatorSchema<String>>,
    pub sender: Option<OperatorSchema<String>>,
    pub receiver: Option<OperatorSchema<String>>,
    pub price: Option<OperatorSchema<i64>>,
    pub usd_price: Option<OperatorSchema<BigDecimal>>,
    pub market_name: Option<OperatorSchema<String>>,
    pub market_contract_id: Option<OperatorSchema<String>>,
    pub nft_id: Option<OperatorSchema<Uuid>>,
    pub collection_id: Option<OperatorSchema<Uuid>>,
    pub block_time: Option<OperatorSchema<Date>>,
    pub block_height: Option<OperatorSchema<i64>>,
    pub amount: Option<OperatorSchema<i64>>,
    pub collection: Option<Arc<QueryCollectionSchema>>,
    pub nft: Option<QueryNftSchema>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(name = "ActivityOrder", rename_fields = "snake_case")]
pub struct OrderActivitySchema {
    pub id: Option<OrderingType>,
    #[graphql(name = "type")]
    pub tx_type: Option<OrderingType>,
    pub tx_index: Option<OrderingType>,
    pub tx_id: Option<OrderingType>,
    pub sender: Option<OrderingType>,
    pub receiver: Option<OrderingType>,
    pub price: Option<OrderingType>,
    pub usd_price: Option<OrderingType>,
    pub market_name: Option<OrderingType>,
    pub market_contract_id: Option<OrderingType>,
    pub nft_id: Option<OrderingType>,
    pub collection_id: Option<OrderingType>,
    pub block_time: Option<OrderingType>,
    pub block_height: Option<OrderingType>,
    pub amount: Option<OrderingType>,
    pub collection: Option<Arc<OrderCollectionSchema>>,
    pub nft: Option<Arc<OrderNftSchema>>,
}
