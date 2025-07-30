use std::sync::Arc;

use async_graphql::{Context, InputObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{
    database::{Database, IDatabase, activities::IActivities, bids::IBids},
    utils::string_utils,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow)]
pub struct CollectionSchema {
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
    pub total_volume: Option<BigDecimal>,
    pub total_sale: Option<i64>,
    pub total_owner: Option<i64>,
    pub floor: Option<BigDecimal>,
    pub listed: Option<i64>,
}

#[async_graphql::Object]
impl CollectionSchema {
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

    async fn floor(&self) -> Option<String> {
        self.floor.as_ref().map(|e| e.to_string())
    }

    async fn listed(&self) -> Option<i64> {
        self.listed
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

        let res = db.bids().fetch_collection_top_offer(collection_id).await;
        if res.is_err() {
            return None;
        }

        res.unwrap().map(|e| e.to_plain_string())
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

    #[graphql(name = "past_floor")]
    async fn past_floor(&self, ctx: &Context<'_>, interval: Option<String>) -> Option<String> {
        if self.id.is_none() {
            return None;
        }

        let i = string_utils::str_to_pginterval(&interval.unwrap_or_default())
            .expect("Invalid interval");

        let collection_id = self.id.as_ref().unwrap();
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let res = db.activities().fetch_past_floor(collection_id, i).await;
        if res.is_err() {
            None
        } else {
            res.unwrap().map(|e| e.to_plain_string())
        }
    }

    async fn sale(&self, ctx: &Context<'_>, interval: Option<String>) -> Option<CollectionSale> {
        if self.id.is_none() {
            return None;
        }

        let i = string_utils::str_to_pginterval(&interval.unwrap_or_default())
            .expect("Invalid interval");

        let collection_id = self.id.as_ref().unwrap();
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        db.activities().fetch_sale(collection_id, i).await.ok()
    }

    #[graphql(name = "total_volume")]
    async fn total_volume(&self) -> Option<String> {
        self.total_volume.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "total_sale")]
    async fn total_sale(&self) -> Option<i64> {
        self.total_sale
    }

    #[graphql(name = "total_owner")]
    async fn total_owner(&self) -> Option<i64> {
        self.total_owner
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow)]
pub struct CollectionSale {
    pub total: Option<i64>,
    pub volume: Option<BigDecimal>,
    pub volume_usd: Option<BigDecimal>,
}

#[async_graphql::Object]
impl CollectionSale {
    async fn total(&self) -> Option<i64> {
        self.total
    }

    async fn volume(&self) -> Option<String> {
        self.volume.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "volume_usd")]
    async fn volume_usd(&self) -> Option<String> {
        self.volume_usd.as_ref().map(|e| e.to_string())
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterCollectionSchema {
    #[graphql(name = "where")]
    pub where_: Option<CollectionWhereSchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CollectionWhereSchema {
    pub collection_id: Option<String>,
}
