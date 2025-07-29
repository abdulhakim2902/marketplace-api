use std::sync::Arc;

use async_graphql::Context;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::database::{
    Database, IDatabase, activities::IActivities, bids::IBids, listings::IListings, nfts::INfts,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow)]
pub struct Collection {
    pub id: Option<String>,
    pub slug: Option<String>,
    pub supply: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub verified: Option<bool>,
    pub website: Option<String>,
    pub discord: Option<String>,
    pub twitter: Option<String>,
    pub royalty: Option<BigDecimal>,
    pub prev_floor: Option<BigDecimal>,
    pub sales: Option<i64>,
    pub volume: Option<BigDecimal>,
    pub volume_usd: Option<BigDecimal>,
}

#[async_graphql::Object]
impl Collection {
    async fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|e| e.as_str())
    }

    async fn slug(&self) -> Option<&str> {
        self.slug.as_ref().map(|e| e.as_str())
    }

    async fn supply(&self) -> Option<i64> {
        self.supply
    }

    async fn title(&self) -> Option<&str> {
        self.title.as_ref().map(|e| e.as_str())
    }

    async fn description(&self) -> Option<&str> {
        self.description.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "cover_url")]
    async fn cover_url(&self) -> Option<&str> {
        self.cover_url.as_ref().map(|e| e.as_str())
    }

    async fn floor(&self, ctx: &Context<'_>) -> Option<String> {
        if self.id.is_none() {
            return None;
        }

        let collection_id = self.id.as_ref().unwrap();
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let res = db.listings().fetch_collection_floor(collection_id).await;

        if res.is_err() {
            return None;
        }

        res.unwrap().map(|e| e.to_plain_string())
    }

    async fn listed(&self, ctx: &Context<'_>) -> Option<i64> {
        if self.id.is_none() {
            return None;
        }

        let collection_id = self.id.as_ref().unwrap();
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.listings().fetch_total_listed(collection_id).await.ok()
    }

    async fn owners(&self, ctx: &Context<'_>) -> Option<i64> {
        if self.id.is_none() {
            return None;
        }

        let collection_id = self.id.as_ref().unwrap();
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.nfts().fetch_total_owners(collection_id).await.ok()
    }

    #[graphql(name = "top_offer")]
    async fn top_offer(&self, ctx: &Context<'_>) -> Option<String> {
        if self.id.is_none() {
            return None;
        }

        let collection_id = self.id.as_ref().unwrap();
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let res = db.bids().fetch_top_offer(collection_id).await;
        if res.is_err() {
            return None;
        }

        res.unwrap().map(|e| e.to_string())
    }

    async fn verified(&self) -> Option<bool> {
        self.verified
    }

    async fn website(&self) -> Option<&str> {
        self.website.as_ref().map(|e| e.as_str())
    }

    async fn discord(&self) -> Option<&str> {
        self.discord.as_ref().map(|e| e.as_str())
    }

    async fn twitter(&self) -> Option<&str> {
        self.twitter.as_ref().map(|e| e.as_str())
    }

    async fn royalty(&self) -> Option<String> {
        self.royalty.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "prev_floor")]
    async fn prev_floor(&self) -> Option<String> {
        self.prev_floor.as_ref().map(|e| e.to_plain_string())
    }

    async fn sales(&self, ctx: &Context<'_>) -> Option<i64> {
        if self.id.is_none() {
            return None;
        }

        if self.sales.is_none() {
            let collection_id = self.id.as_ref().unwrap();
            let db = ctx
                .data::<Arc<Database>>()
                .expect("Missing database in the context");

            db.activities().fetch_sales(collection_id).await.ok()
        } else {
            self.sales
        }
    }

    async fn volume(&self, ctx: &Context<'_>) -> Option<String> {
        if self.id.is_none() {
            return None;
        }

        if self.volume.is_none() {
            let collection_id = self.id.as_ref().unwrap();
            let db = ctx
                .data::<Arc<Database>>()
                .expect("Missing database in the context");

            let res = db.activities().fetch_volume(collection_id).await;
            if res.is_err() {
                None
            } else {
                res.unwrap().map(|e| e.to_plain_string())
            }
        } else {
            self.volume.as_ref().map(|e| e.to_plain_string())
        }
    }

    #[graphql(name = "volume_usd")]
    async fn volume_usd(&self, ctx: &Context<'_>) -> Option<String> {
        if self.id.is_none() {
            return None;
        }

        if self.volume.is_none() {
            let collection_id = self.id.as_ref().unwrap();
            let db = ctx
                .data::<Arc<Database>>()
                .expect("Missing database in the context");

            let res = db.activities().fetch_volume_usd(collection_id).await;
            if res.is_err() {
                None
            } else {
                res.unwrap().map(|e| e.to_plain_string())
            }
        } else {
            self.volume_usd.as_ref().map(|e| e.to_plain_string())
        }
    }

    #[graphql(name = "sales_24h")]
    async fn sales_24h(&self, ctx: &Context<'_>) -> Option<i64> {
        if self.id.is_none() {
            return None;
        }

        let collection_id = self.id.as_ref().unwrap();
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.activities().fetch_sales_24h(collection_id).await.ok()
    }

    #[graphql(name = "volume_24h")]
    async fn volume_24h(&self, ctx: &Context<'_>) -> Option<String> {
        if self.id.is_none() {
            return None;
        }

        let collection_id = self.id.as_ref().unwrap();
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let res = db.activities().fetch_volume_24h(collection_id).await;
        if res.is_err() {
            return None;
        }

        res.unwrap().map(|e| e.to_string())
    }

    #[graphql(name = "volume_usd_24h")]
    async fn volume_usd_24h(&self, ctx: &Context<'_>) -> Option<String> {
        if self.id.is_none() {
            return None;
        }

        let collection_id = self.id.as_ref().unwrap();
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let res = db.activities().fetch_volume_usd_24h(collection_id).await;
        if res.is_err() {
            return None;
        }

        res.unwrap().map(|e| e.to_string())
    }
}
