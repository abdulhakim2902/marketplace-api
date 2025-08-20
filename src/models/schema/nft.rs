use std::sync::Arc;

use crate::{
    database::{
        Database, IDatabase, activities::IActivities, attributes::IAttributes, bids::IBids,
        collections::Collections, listings::IListings,
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
        collection::{CollectionSchema, OrderCollectionSchema, QueryCollectionSchema},
        fetch_nft_top_offer,
        listing::{DistinctListingSchema, ListingSchema, OrderListingSchema, QueryListingSchema},
    },
};
use async_graphql::{
    ComplexObject, Context, Enum, InputObject, SimpleObject, dataloader::DataLoader,
};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(complex, name = "Nft", rename_fields = "snake_case")]
pub struct NftSchema {
    pub id: Uuid,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub collection_id: Option<Uuid>,
    pub burned: Option<bool>,
    pub properties: Option<serde_json::Value>,
    pub description: Option<String>,
    #[graphql(name = "media_url")]
    pub image_url: Option<String>,
    pub token_id: Option<String>,
    pub animation_url: Option<String>,
    pub avatar_url: Option<String>,
    pub external_url: Option<String>,
    pub youtube_url: Option<String>,
    pub background_color: Option<String>,
    pub royalty: Option<BigDecimal>,
    #[graphql(visible = false)]
    pub version: Option<String>,
    pub ranking: Option<i64>,
    pub rarity: Option<BigDecimal>,
}

#[ComplexObject]
impl NftSchema {
    #[graphql(name = "top_offer")]
    async fn top_offer(&self, ctx: &Context<'_>) -> Option<String> {
        fetch_nft_top_offer(ctx, &self.id.to_string()).await
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
        query.nft_id = Some(operator);

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
        query.nft_id = Some(operator);

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
        query.nft_id = Some(operator);

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
        query.nft_id = Some(operator);

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

    async fn listings(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctListingSchema>,
        #[graphql(name = "where")] query: Option<QueryListingSchema>,
        #[graphql(name = "order_by")] order: Option<OrderListingSchema>,
    ) -> Vec<ListingSchema> {
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
        query.nft_id = Some(operator);

        db.listings()
            .fetch_listings(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch nfts")
    }

    #[graphql(name = "listings_aggregate")]
    async fn listings_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctListingSchema>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryListingSchema>,
        #[graphql(name = "order_by")] order: Option<OrderListingSchema>,
    ) -> AggregateSchema<ListingSchema> {
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
        query.nft_id = Some(operator);

        let total = db
            .listings()
            .fetch_total_listings(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch total listings");

        let nodes = db
            .listings()
            .fetch_listings(&distinct, limit, offset, &query, &order)
            .await
            .expect("Failed to fetch listings");

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
        query.nft_id = Some(operator);

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
        query.nft_id = Some(operator);

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
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(name = "NftQuery", rename_fields = "snake_case")]
pub struct QueryNftSchema {
    #[graphql(name = "_or")]
    pub _or: Option<Box<QueryNftSchema>>,
    #[graphql(name = "_and")]
    pub _and: Option<Box<QueryNftSchema>>,
    #[graphql(name = "_not")]
    pub _not: Option<Box<QueryNftSchema>>,
    pub id: Option<OperatorSchema<Uuid>>,
    pub name: Option<OperatorSchema<String>>,
    pub owner: Option<OperatorSchema<String>>,
    pub collection_id: Option<OperatorSchema<Uuid>>,
    pub burned: Option<OperatorSchema<bool>>,
    #[graphql(visible = false)]
    pub properties: Option<serde_json::Value>,
    pub description: Option<OperatorSchema<String>>,
    #[graphql(name = "media_url")]
    pub image_url: Option<OperatorSchema<String>>,
    pub token_id: Option<OperatorSchema<String>>,
    pub animation_url: Option<OperatorSchema<String>>,
    pub avatar_url: Option<OperatorSchema<String>>,
    pub external_url: Option<OperatorSchema<String>>,
    pub youtube_url: Option<OperatorSchema<String>>,
    pub background_color: Option<OperatorSchema<String>>,
    pub royalty: Option<OperatorSchema<BigDecimal>>,
    #[graphql(visible = false)]
    pub version: Option<OperatorSchema<String>>,
    pub ranking: Option<OperatorSchema<i64>>,
    pub rarity: Option<OperatorSchema<BigDecimal>>,
    pub collection: Option<Arc<QueryCollectionSchema>>,
    pub activity: Option<Arc<QueryActivitySchema>>,
    pub attribute: Option<Arc<QueryAttributeSchema>>,
    pub listing: Option<Arc<QueryListingSchema>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(name = "NftOrderBy", rename_fields = "snake_case")]
pub struct OrderNftSchema {
    pub id: Option<OrderingType>,
    pub name: Option<OrderingType>,
    pub owner: Option<OrderingType>,
    pub collection_id: Option<OrderingType>,
    pub burned: Option<OrderingType>,
    #[graphql(visible = false)]
    pub properties: Option<serde_json::Value>,
    pub description: Option<OrderingType>,
    #[graphql(name = "media_url")]
    pub image_url: Option<OrderingType>,
    pub token_id: Option<OrderingType>,
    pub animation_url: Option<OrderingType>,
    pub avatar_url: Option<OrderingType>,
    pub external_url: Option<OrderingType>,
    pub youtube_url: Option<OrderingType>,
    pub background_color: Option<OrderingType>,
    pub royalty: Option<OrderingType>,
    #[graphql(visible = false)]
    pub version: Option<OrderingType>,
    pub ranking: Option<OrderingType>,
    pub rarity: Option<OrderingType>,
    pub collection: Option<OrderCollectionSchema>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[graphql(name = "NftDistinctOn", rename_items = "snake_case")]
pub enum DistinctNftSchema {
    Id,
    Name,
    Owner,
    CollectionId,
    Burned,
    Description,
    #[graphql(name = "media_url")]
    ImageUrl,
    TokenId,
    AnimationUrl,
    AvatarUrl,
    ExternalUrl,
    YoutubeUrl,
    BackgroundColor,
    Royalty,
    Version,
    Ranking,
    Rarity,
}

impl Default for DistinctNftSchema {
    fn default() -> Self {
        Self::Id
    }
}
