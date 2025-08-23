pub mod attribute;
pub mod holder;
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
        AggregateFieldsSchema, AggregateSchema, OperatorSchema, OrderingType,
        activity::{
            ActivitySchema, AggregateActivitySchema, DistinctActivitySchema, OrderActivitySchema,
            QueryActivitySchema,
        },
        attribute::{
            AggregateAttributeSchema, AttributeSchema, DistinctAttributeSchema,
            OrderAttributeSchema, QueryAttributeSchema,
        },
        bid::{AggregateBidSchema, BidSchema, DistinctBidSchema, OrderBidSchema, QueryBidSchema},
        get_aggregate_selection,
        nft::{AggregateNftSchema, DistinctNftSchema, NftSchema, OrderNftSchema, QueryNftSchema},
    },
};
use async_graphql::{
    ComplexObject, Context, Enum, FieldError, FieldResult, InputObject, SimpleObject,
};
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
    pub creator_address: Option<String>,
    #[graphql(visible = false)]
    pub table_handle: Option<String>,
}

#[ComplexObject]
impl CollectionSchema {
    async fn activities(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctActivitySchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryActivitySchema,
        #[graphql(default, name = "order_by")] order: OrderActivitySchema,
    ) -> FieldResult<Vec<ActivitySchema>> {
        let mut query = query;
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .activities()
            .fetch_activities(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "activities_aggregate")]
    async fn activities_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctActivitySchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryActivitySchema,
        #[graphql(default, name = "order_by")] order: OrderActivitySchema,
    ) -> FieldResult<AggregateSchema<AggregateActivitySchema, ActivitySchema>> {
        let mut query = query;
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let selection = get_aggregate_selection(ctx);

        let aggregate = db
            .activities()
            .fetch_aggregate_activities(&selection.aggregate, &query, distinct.as_ref())
            .await?;

        if selection.nodes.is_empty() {
            return Ok(AggregateSchema {
                aggregate,
                nodes: Vec::new(),
            });
        }

        let nodes = db
            .activities()
            .fetch_activities(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema { aggregate, nodes })
    }

    async fn attributes(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctAttributeSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryAttributeSchema,
        #[graphql(default, name = "order_by")] order: OrderAttributeSchema,
    ) -> FieldResult<Vec<AttributeSchema>> {
        let mut query = query;
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .attributes()
            .fetch_attributes(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "attributes_aggregate")]
    async fn attributes_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctAttributeSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryAttributeSchema,
        #[graphql(default, name = "order_by")] order: OrderAttributeSchema,
    ) -> FieldResult<AggregateSchema<AggregateAttributeSchema, AttributeSchema>> {
        let mut query = query;
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let selection = get_aggregate_selection(ctx);

        let aggregate = db
            .attributes()
            .fetch_aggregate_attributes(&selection.aggregate, &query, distinct.as_ref())
            .await?;

        if selection.nodes.is_empty() {
            return Ok(AggregateSchema {
                aggregate,
                nodes: Vec::new(),
            });
        }

        let nodes = db
            .attributes()
            .fetch_attributes(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema { aggregate, nodes })
    }

    async fn bids(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctBidSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryBidSchema,
        #[graphql(default, name = "order_by")] order: OrderBidSchema,
    ) -> FieldResult<Vec<BidSchema>> {
        let mut query = query;
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .bids()
            .fetch_bids(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "bids_aggregate")]
    async fn bids_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctBidSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryBidSchema,
        #[graphql(default, name = "order_by")] order: OrderBidSchema,
    ) -> FieldResult<AggregateSchema<AggregateBidSchema, BidSchema>> {
        let mut query = query;
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let selection = get_aggregate_selection(ctx);

        let aggregate = db
            .bids()
            .fetch_aggregate_bids(&selection.aggregate, &query, distinct.as_ref())
            .await?;

        if selection.nodes.is_empty() {
            return Ok(AggregateSchema {
                aggregate,
                nodes: Vec::new(),
            });
        }

        let nodes = db
            .bids()
            .fetch_bids(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema { aggregate, nodes })
    }

    async fn nfts(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctNftSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryNftSchema,
        #[graphql(default, name = "order_by")] order: OrderNftSchema,
    ) -> FieldResult<Vec<NftSchema>> {
        let mut query = query;
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .nfts()
            .fetch_nfts(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "nfts_aggregate")]
    async fn nfts_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctNftSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryNftSchema,
        #[graphql(default, name = "order_by")] order: OrderNftSchema,
    ) -> FieldResult<AggregateSchema<AggregateNftSchema, NftSchema>> {
        let mut query = query;
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.collection_id = Some(operator);

        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let selection = get_aggregate_selection(ctx);

        let aggregate = db
            .nfts()
            .fetch_aggregate_nfts(&selection.aggregate, &query, distinct.as_ref())
            .await?;

        if selection.nodes.is_empty() {
            return Ok(AggregateSchema {
                aggregate,
                nodes: Vec::new(),
            });
        }

        let nodes = db
            .nfts()
            .fetch_nfts(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema { aggregate, nodes })
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, SimpleObject)]
#[graphql(name = "AggregateCollectionFields", rename_fields = "snake_case")]
pub struct AggregateCollectionFieldsSchema {
    pub supply: Option<BigDecimal>,
    pub floor: Option<BigDecimal>,
    pub volume: Option<BigDecimal>,
    pub volume_usd: Option<BigDecimal>,
}

pub type AggregateCollectionSchema = AggregateFieldsSchema<AggregateCollectionFieldsSchema>;
