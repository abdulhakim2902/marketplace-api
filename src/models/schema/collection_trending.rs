use std::sync::Arc;

use crate::{
    database::{Database, IDatabase, collections::ICollections, nfts::INfts},
    models::schema::{collection::CollectionSchema, nft::NftSchema},
};
use async_graphql::Context;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CollectionTrendingSchema {
    pub nft_id: Option<String>,
    pub collection_id: Option<String>,
    pub tx_frequency: Option<i64>,
    pub last_price: Option<BigDecimal>,
}

#[async_graphql::Object]
impl CollectionTrendingSchema {
    #[graphql(name = "collection_id")]
    async fn collection_id(&self) -> Option<&str> {
        self.collection_id.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "nft_id")]
    async fn nft_id(&self) -> Option<&str> {
        self.nft_id.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "tx_frequency")]
    async fn tx_frequency(&self) -> Option<i64> {
        self.tx_frequency
    }

    #[graphql(name = "last_price")]
    async fn last_price(&self) -> Option<String> {
        self.last_price.as_ref().map(|e| e.to_string())
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

    async fn nft(&self, ctx: &Context<'_>) -> Option<NftSchema> {
        if self.nft_id.is_none() {
            return None;
        }

        let collection_id: &String = self.collection_id.as_ref().unwrap();
        let nft_id = self.nft_id.as_ref().unwrap();
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let res = db
            .nfts()
            .fetch_nfts(
                Some(nft_id.to_string()),
                Some(collection_id.to_string()),
                1,
                0,
            )
            .await;

        if res.is_err() {
            return None;
        }

        res.unwrap().first().cloned()
    }
}
