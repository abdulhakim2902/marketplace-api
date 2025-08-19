use std::sync::Arc;

use crate::{
    database::{Database, IDatabase, nfts::Nfts},
    models::{
        marketplace::APT_DECIMAL,
        schema::{
            Date, OperatorSchema, OrderingType, fetch_token_price,
            nft::{NftSchema, OrderNftSchema, QueryNftSchema},
        },
    },
};
use async_graphql::{
    ComplexObject, Context, Enum, InputObject, SimpleObject, dataloader::DataLoader,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject, FromRow)]
#[graphql(complex, name = "Listing", rename_fields = "snake_case")]
pub struct ListingSchema {
    pub id: Uuid,
    pub block_height: Option<i64>,
    pub block_time: Option<DateTime<Utc>>,
    pub market_contract_id: Option<String>,
    pub listed: Option<bool>,
    pub market_name: Option<String>,
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
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(name = "ListingQuery", rename_fields = "snake_case")]
pub struct QueryListingSchema {
    #[graphql(name = "_or")]
    pub _or: Option<Arc<QueryListingSchema>>,
    #[graphql(name = "_and")]
    pub _and: Option<Arc<QueryListingSchema>>,
    #[graphql(name = "_not")]
    pub _not: Option<Arc<QueryListingSchema>>,
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
#[graphql(name = "ListingOrderBy", rename_fields = "snake_case")]
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
    pub nft: Option<OrderNftSchema>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[graphql(name = "ListingDistinctOn", rename_items = "snake_case")]
pub enum DistinctListingSchema {
    Id,
    BlockHeight,
    BlockTime,
    MarketContractId,
    Listed,
    MarketName,
    CollectionId,
    NftId,
    Nonce,
    Price,
    Seller,
    TxIndex,
}

impl Default for DistinctListingSchema {
    fn default() -> Self {
        Self::Id
    }
}
