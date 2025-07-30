use std::sync::Arc;

use async_graphql::{Context, InputObject};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    database::{Database, IDatabase, collections::ICollections, nfts::INfts},
    models::schema::{collection::CollectionSchema, nft::NftSchema},
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ActivitySchema {
    pub tx_type: Option<String>,
    pub tx_index: i64,
    pub tx_id: String,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub price: Option<BigDecimal>,
    pub usd_price: Option<BigDecimal>,
    pub market_name: Option<String>,
    pub market_contract_id: Option<String>,
    pub nft_id: Option<String>,
    pub collection_id: Option<String>,
    pub block_time: Option<DateTime<Utc>>,
    pub block_height: Option<i64>,
    pub amount: Option<i64>,
}

#[async_graphql::Object]
impl ActivitySchema {
    #[graphql(name = "type")]
    async fn tx_type(&self) -> Option<&str> {
        self.tx_type.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "tx_index")]
    async fn tx_index(&self) -> i64 {
        self.tx_index
    }

    #[graphql(name = "tx_id")]
    async fn tx_id(&self) -> &str {
        &self.tx_id
    }

    async fn sender(&self) -> Option<&str> {
        self.sender.as_ref().map(|e| e.as_str())
    }

    async fn receiver(&self) -> Option<&str> {
        self.receiver.as_ref().map(|e| e.as_str())
    }

    async fn price(&self) -> Option<String> {
        self.price.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "usd_price")]
    async fn usd_price(&self) -> Option<String> {
        self.usd_price.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "market_name")]
    async fn market_name(&self) -> Option<&str> {
        self.market_name.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "market_contract_id")]
    async fn market_contract_id(&self) -> Option<&str> {
        self.market_contract_id.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "nft_id")]
    async fn nft_id(&self) -> Option<&str> {
        self.nft_id.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "collection_id")]
    async fn collection_id(&self) -> Option<String> {
        self.collection_id.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "block_time")]
    async fn block_time(&self) -> Option<String> {
        self.block_time.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "block_height")]
    async fn block_height(&self) -> Option<i64> {
        self.block_height
    }

    #[graphql(name = "amount")]
    async fn amount(&self) -> Option<i64> {
        self.amount
    }

    async fn nft(&self, ctx: &Context<'_>) -> Option<NftSchema> {
        if self.nft_id.is_none() {
            return None;
        }

        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let res = db
            .nfts()
            .fetch_nfts(self.nft_id.clone(), self.collection_id.clone(), 1, 0)
            .await;

        if res.is_err() {
            return None;
        }

        res.unwrap().first().cloned()
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        if self.collection_id.is_none() {
            return None;
        }

        let collection_id = self.collection_id.as_ref().unwrap();
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let res = db
            .collections()
            .fetch_collections(Some(collection_id.to_string()), 1, 0)
            .await;

        if res.is_err() {
            return None;
        }

        res.unwrap().first().cloned()
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterActivitySchema {
    #[graphql(name = "where")]
    pub where_: Option<ActivityWhereSchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct ActivityWhereSchema {
    pub wallet_address: Option<String>,
    pub collection_id: Option<String>,
    pub nft_id: Option<String>,
}
