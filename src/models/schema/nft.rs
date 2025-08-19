use std::sync::Arc;

use crate::{
    database::{
        Database, IDatabase, activities::IActivities, attributes::IAttributes, bids::IBids,
        collections::Collections, listings::IListings,
    },
    models::schema::{
        OperatorSchema, OrderingType,
        activity::{ActivitySchema, OrderActivitySchema, QueryActivitySchema},
        attribute::{AttributeSchema, OrderAttributeSchema, QueryAttributeSchema},
        bid::{BidSchema, OrderBidSchema, QueryBidSchema},
        collection::{CollectionSchema, OrderCollectionSchema, QueryCollectionSchema},
        fetch_nft_top_offer,
        listing::{ListingSchema, OrderListingSchema, QueryListingSchema},
    },
};
use async_graphql::{ComplexObject, Context, InputObject, SimpleObject, dataloader::DataLoader};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
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
        #[graphql(name = "where")] query: Option<QueryAttributeSchema>,
        #[graphql(name = "order_by")] order: Option<OrderAttributeSchema>,
    ) -> Vec<AttributeSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        let mut query = query.unwrap_or_default();
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.nft_id = Some(operator);

        db.attributes()
            .fetch_attributes(limit, offset, query, order)
            .await
            .expect("Failed to fetch attributes")
    }

    async fn activities(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryActivitySchema>,
        #[graphql(name = "order_by")] order: Option<OrderActivitySchema>,
    ) -> Vec<ActivitySchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        let mut query = query.unwrap_or_default();
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.nft_id = Some(operator);

        db.activities()
            .fetch_activities(limit, offset, query, order)
            .await
            .expect("Failed to fetch activities")
    }

    async fn listings(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryListingSchema>,
        #[graphql(name = "order_by")] order: Option<OrderListingSchema>,
    ) -> Vec<ListingSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        let mut query = query.unwrap_or_default();
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.nft_id = Some(operator);

        db.listings()
            .fetch_listings(limit, offset, query, order)
            .await
            .expect("Failed to fetch nfts")
    }

    async fn bids(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        offset: Option<i64>,
        #[graphql(name = "where")] query: Option<QueryBidSchema>,
        #[graphql(name = "order_by")] order: Option<OrderBidSchema>,
    ) -> Vec<BidSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);
        let order = order.unwrap_or_default();

        let mut query = query.unwrap_or_default();
        let mut operator = OperatorSchema::<Uuid>::default();

        operator._eq = Some(self.id);
        query.nft_id = Some(operator);

        db.bids()
            .fetch_bids(limit, offset, query, order)
            .await
            .expect("Failed to fetch bids")
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
    // pub properties: Option<serde_json::Value>,
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
    // pub properties: Option<serde_json::Value>,
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
    pub version: Option<OrderingType>,
    pub ranking: Option<OrderingType>,
    pub rarity: Option<OrderingType>,
    pub collection: Option<OrderCollectionSchema>,
}
