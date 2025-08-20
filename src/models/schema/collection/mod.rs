pub mod attribute;
pub mod nft_change;
pub mod nft_distribution;
pub mod nft_holder;
pub mod profit_leaderboard;
pub mod stat;
pub mod top_wallet;
pub mod trending;
pub mod trending_nft;

use std::sync::Arc;

use crate::{
    database::{
        Database, IDatabase, activities::IActivities, attributes::IAttributes, bids::IBids,
        nfts::INfts,
    },
    models::schema::{
        AggregateSchema, OperatorSchema, OrderingType,
        activity::{
            ActivitySchema, DistinctActivitySchema, OrderActivitySchema, QueryActivitySchema,
        },
        attribute::{
            AttributeSchema, DistinctAttributeSchema, OrderAttributeSchema, QueryAttributeSchema,
        },
        bid::{BidSchema, DistinctBidSchema, OrderBidSchema, QueryBidSchema},
        fetch_total_collection_offer, fetch_total_collection_trait,
        nft::{DistinctNftSchema, NftSchema, OrderNftSchema, QueryNftSchema},
    },
};
use async_graphql::{ComplexObject, Context, Enum, InputObject, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(complex, name = "collection", rename_fields = "snake_case")]
pub struct CollectionSchema {
    pub id: Uuid,
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
    pub floor: Option<i64>,
    pub volume: Option<i64>,
    pub volume_usd: Option<BigDecimal>,
    #[graphql(visible = false)]
    pub listed: Option<i64>,
    #[graphql(visible = false)]
    pub sales: Option<i64>,
    #[graphql(visible = false)]
    pub owners: Option<i64>,
    #[graphql(visible = false)]
    pub creator_address: Option<String>,
    #[graphql(visible = false)]
    pub table_handle: Option<String>,
}

#[ComplexObject]
impl CollectionSchema {
    #[graphql(name = "total_trait")]
    async fn total_trait(&self, ctx: &Context<'_>) -> Option<i64> {
        fetch_total_collection_trait(ctx, Some(self.id.to_string())).await
    }

    #[graphql(name = "total_offer")]
    async fn total_offer(&self, ctx: &Context<'_>) -> Option<String> {
        fetch_total_collection_offer(ctx, Some(self.id.to_string())).await
    }

    async fn attributes(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctAttributeSchema>,
        #[graphql(name = "where")] query: Option<QueryAttributeSchema>,
        #[graphql(name = "order_by")] order: Option<OrderAttributeSchema>,
    ) -> Vec<AttributeSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        let mut query = query.unwrap_or_default();
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        db.attributes()
            .fetch_attributes(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch attributes")
    }

    #[graphql(name = "attributes_aggregate")]
    async fn attributes_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctAttributeSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryAttributeSchema>,
        #[graphql(name = "order_by")] order: Option<OrderAttributeSchema>,
    ) -> AggregateSchema<AttributeSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        let mut query = query.unwrap_or_default();
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        let total = db
            .attributes()
            .fetch_total_attributes(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch total ttributes");

        let nodes = db
            .attributes()
            .fetch_attributes(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch attributes");

        AggregateSchema::new(total, nodes)
    }

    async fn activities(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctActivitySchema>,
        #[graphql(name = "where")] query: Option<QueryActivitySchema>,
        #[graphql(name = "order_by")] order: Option<OrderActivitySchema>,
    ) -> Vec<ActivitySchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        let mut query = query.unwrap_or_default();
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        db.activities()
            .fetch_activities(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch activities")
    }

    #[graphql(name = "activities_aggregate")]
    async fn activities_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctActivitySchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryActivitySchema>,
        #[graphql(name = "order_by")] order: Option<OrderActivitySchema>,
    ) -> AggregateSchema<ActivitySchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        let mut query = query.unwrap_or_default();
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        let total = db
            .activities()
            .fetch_total_activities(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch total activities");

        let nodes = db
            .activities()
            .fetch_activities(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch activities");

        AggregateSchema::new(total, nodes)
    }

    async fn bids(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctBidSchema>,
        #[graphql(name = "where")] query: Option<QueryBidSchema>,
        #[graphql(name = "order_by")] order: Option<OrderBidSchema>,
    ) -> Vec<BidSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        let mut query = query.unwrap_or_default();
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        db.bids()
            .fetch_bids(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch bids")
    }

    #[graphql(name = "bids_aggregate")]
    async fn bids_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctBidSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryBidSchema>,
        #[graphql(name = "order_by")] order: Option<OrderBidSchema>,
    ) -> AggregateSchema<BidSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        let mut query = query.unwrap_or_default();
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        let total = db
            .bids()
            .fetch_total_bids(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch total bids");

        let nodes = db
            .bids()
            .fetch_bids(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch bids");

        AggregateSchema::new(total, nodes)
    }

    async fn nfts(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctNftSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryNftSchema>,
        #[graphql(name = "order_by")] order: Option<OrderNftSchema>,
    ) -> Vec<NftSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        let mut query = query.unwrap_or_default();
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        db.nfts()
            .fetch_nfts(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch nfts")
    }

    #[graphql(name = "nfts_aggregate")]
    async fn nfts_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctNftSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryNftSchema>,
        #[graphql(name = "order_by")] order: Option<OrderNftSchema>,
    ) -> AggregateSchema<NftSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let distinct = distinct.unwrap_or_default();
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        let mut query = query.unwrap_or_default();
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        let total = db
            .nfts()
            .fetch_total_nfts(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch total nfts");

        let nodes = db
            .nfts()
            .fetch_nfts(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch nfts");

        AggregateSchema::new(total, nodes)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(name = "CollectionQuery", rename_fields = "snake_case")]
pub struct QueryCollectionSchema {
    #[graphql(name = "_or")]
    pub _or: Option<Arc<QueryCollectionSchema>>,
    #[graphql(name = "_and")]
    pub _and: Option<Arc<QueryCollectionSchema>>,
    #[graphql(name = "_not")]
    pub _not: Option<Arc<QueryCollectionSchema>>,
    pub id: Option<OperatorSchema<Uuid>>,
    pub slug: Option<OperatorSchema<String>>,
    pub supply: Option<OperatorSchema<i64>>,
    pub title: Option<OperatorSchema<String>>,
    pub description: Option<OperatorSchema<String>>,
    pub cover_url: Option<OperatorSchema<String>>,
    pub verified: Option<OperatorSchema<bool>>,
    pub website: Option<OperatorSchema<String>>,
    pub discord: Option<OperatorSchema<String>>,
    pub twitter: Option<OperatorSchema<String>>,
    pub royalty: Option<OperatorSchema<BigDecimal>>,
    pub floor: Option<OperatorSchema<i64>>,
    pub volume: Option<OperatorSchema<i64>>,
    pub volume_usd: Option<OperatorSchema<BigDecimal>>,
    pub activity: Option<Arc<QueryActivitySchema>>,
    pub attribute: Option<Arc<QueryAttributeSchema>>,
    pub bid: Option<Arc<QueryBidSchema>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(name = "CollectionOrderBy", rename_fields = "snake_case")]
pub struct OrderCollectionSchema {
    pub id: Option<OrderingType>,
    pub slug: Option<OrderingType>,
    pub supply: Option<OrderingType>,
    pub title: Option<OrderingType>,
    pub description: Option<OrderingType>,
    pub cover_url: Option<OrderingType>,
    pub verified: Option<OrderingType>,
    pub website: Option<OrderingType>,
    pub discord: Option<OrderingType>,
    pub twitter: Option<OrderingType>,
    pub royalty: Option<OrderingType>,
    pub floor: Option<OrderingType>,
    pub volume: Option<OrderingType>,
    pub volume_usd: Option<OrderingType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[graphql(name = "CollectionDistinctOn", rename_items = "snake_case")]
pub enum DistinctCollectionSchema {
    Id,
    Slug,
    Supply,
    Title,
    Description,
    CoverUrl,
    Verified,
    Website,
    Discord,
    Twitter,
    Royalty,
    Floor,
    Volume,
    VolumeUsd,
}

impl Default for DistinctCollectionSchema {
    fn default() -> Self {
        Self::Id
    }
}
