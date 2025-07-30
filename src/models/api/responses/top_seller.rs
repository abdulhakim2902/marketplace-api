use std::sync::Arc;

use crate::{
    database::{Database, IDatabase, collections::ICollections},
    models::api::responses::collection::Collection,
};
use async_graphql::Context;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TopSeller {
    pub collection_id: Option<String>,
    pub seller: Option<String>,
    pub sold: Option<i64>,
    pub volume: Option<BigDecimal>,
}

#[async_graphql::Object]
impl TopSeller {
    #[graphql(name = "collection_id")]
    async fn collection_id(&self) -> Option<&str> {
        self.collection_id.as_ref().map(|e| e.as_str())
    }

    async fn sold(&self) -> Option<i64> {
        self.sold
    }

    async fn seller(&self) -> Option<&str> {
        self.seller.as_ref().map(|e| e.as_str())
    }

    async fn volume(&self) -> Option<String> {
        self.volume.as_ref().map(|e| e.to_string())
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<Collection> {
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
