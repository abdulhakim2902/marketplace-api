use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, SimpleObject, dataloader::DataLoader};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    database::{Database, IDatabase, collections::Collections},
    models::schema::{
        OperatorSchema, OrderingType,
        collection::{CollectionSchema, QueryCollectionSchema},
        nft::QueryNftSchema,
    },
};

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct AttributeSchema {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub nft_id: Uuid,
    #[graphql(name = "type")]
    pub attr_type: String,
    pub value: String,
    pub rarity: Option<BigDecimal>,
    pub score: Option<BigDecimal>,
}

#[ComplexObject]
impl AttributeSchema {
    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        let db = ctx
            .data::<Arc<Database>>()
            .expect("Missing database in the context");

        let data_loader = DataLoader::new(
            Collections::new(Arc::new(db.get_pool().clone())),
            tokio::spawn,
        );

        data_loader
            .load_one(self.collection_id)
            .await
            .ok()
            .flatten()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct QueryAttributeSchema {
    #[graphql(name = "_or")]
    pub _or: Option<Arc<QueryAttributeSchema>>,
    #[graphql(name = "_and")]
    pub _and: Option<Arc<QueryAttributeSchema>>,
    #[graphql(name = "_not")]
    pub _not: Option<Arc<QueryAttributeSchema>>,
    pub id: Option<OperatorSchema<Uuid>>,
    pub collection_id: Option<OperatorSchema<Uuid>>,
    pub nft_id: Option<OperatorSchema<Uuid>>,
    #[graphql(name = "type")]
    pub attr_type: Option<OperatorSchema<String>>,
    pub value: Option<OperatorSchema<String>>,
    pub rarity: Option<OperatorSchema<BigDecimal>>,
    pub score: Option<OperatorSchema<BigDecimal>>,
    pub collection: Option<Arc<QueryCollectionSchema>>,
    // TODO:
    pub nft: Option<QueryNftSchema>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct OrderAttributeSchema {
    pub id: Option<OrderingType>,
    pub collection_id: Option<OrderingType>,
    pub nft_id: Option<OrderingType>,
    #[graphql(name = "type")]
    pub attr_type: Option<OrderingType>,
    pub value: Option<OrderingType>,
    pub rarity: Option<OrderingType>,
    pub score: Option<OrderingType>,
}
