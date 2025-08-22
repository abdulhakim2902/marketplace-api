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
        listing::{DistinctListingSchema, ListingSchema, OrderListingSchema, QueryListingSchema},
    },
};
use async_graphql::{
    ComplexObject, Context, Enum, FieldError, FieldResult, InputObject, SimpleObject,
    dataloader::DataLoader,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
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
    pub token_id: Option<String>,
    #[graphql(visible = false)]
    pub media_url: Option<String>,
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
    #[graphql(visible = false)]
    pub uri: Option<String>,
    #[graphql(visible = false)]
    pub updated_at: DateTime<Utc>,
}

#[ComplexObject]
impl NftSchema {
    #[graphql(name = "image_url")]
    async fn image_url(&self) -> Option<&str> {
        if self.media_url.is_some() {
            self.media_url.as_ref().map(|e| e.as_str())
        } else {
            self.uri.as_ref().map(|e| e.as_str())
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
        query.nft_id = Some(operator);

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
    ) -> FieldResult<AggregateSchema<AttributeSchema>> {
        let mut query = query;
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.nft_id = Some(operator);

        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let total = db
            .attributes()
            .fetch_total_attributes(&query, distinct.as_ref())
            .await
            .map_err(|e| FieldError::from(e))?;

        let nodes = db
            .attributes()
            .fetch_attributes(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema::new(total, nodes))
    }

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
        query.nft_id = Some(operator);

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
    ) -> FieldResult<AggregateSchema<ActivitySchema>> {
        let mut query = query;
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.nft_id = Some(operator);

        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let total = db
            .activities()
            .fetch_total_activities(&query, distinct.as_ref())
            .await
            .map_err(|e| FieldError::from(e))?;

        let nodes = db
            .activities()
            .fetch_activities(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema::new(total, nodes))
    }

    async fn listings(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctListingSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryListingSchema,
        #[graphql(default, name = "order_by")] order: OrderListingSchema,
    ) -> FieldResult<Vec<ListingSchema>> {
        let mut query = query;
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.nft_id = Some(operator);

        ctx.data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?
            .listings()
            .fetch_listings(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))
    }

    #[graphql(name = "listings_aggregate")]
    async fn listings_aggregate(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "distinct_on")] distinct: Option<DistinctListingSchema>,
        #[graphql(default = 10)] limit: i64,
        #[graphql(default = 0)] offset: i64,
        #[graphql(default, name = "where")] query: QueryListingSchema,
        #[graphql(default, name = "order_by")] order: OrderListingSchema,
    ) -> FieldResult<AggregateSchema<ListingSchema>> {
        let mut query = query;
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.nft_id = Some(operator);

        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let total = db
            .listings()
            .fetch_total_listings(&query, distinct.as_ref())
            .await
            .map_err(|e| FieldError::from(e))?;

        let nodes = db
            .listings()
            .fetch_listings(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema::new(total, nodes))
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
        query.nft_id = Some(operator);

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
    ) -> FieldResult<AggregateSchema<BidSchema>> {
        let mut query = query;
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.nft_id = Some(operator);

        let db = ctx
            .data::<Arc<Database>>()
            .map_err(|e| FieldError::from(e))?;

        let total = db
            .bids()
            .fetch_total_bids(&query, distinct.as_ref())
            .await
            .map_err(|e| FieldError::from(e))?;

        let nodes = db
            .bids()
            .fetch_bids(&query, &order, distinct.as_ref(), limit, offset)
            .await
            .map_err(|e| FieldError::from(e))?;

        Ok(AggregateSchema::new(total, nodes))
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
    pub media_url: Option<OperatorSchema<String>>,
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
    pub media_url: Option<OrderingType>,
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
    MediaUrl,
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
