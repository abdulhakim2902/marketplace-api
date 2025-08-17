use std::sync::Arc;

use crate::{
    database::{Database, IDatabase, collections::Collections},
    models::schema::{collection::CollectionSchema, fetch_nft, nft::NftSchema},
};
use async_graphql::{ComplexObject, Context, SimpleObject, dataloader::DataLoader};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct TrendingNftSchema {
    pub nft_id: Uuid,
    pub collection_id: Option<Uuid>,
    pub tx_frequency: Option<i64>,
    pub last_price: Option<i64>,
}

#[ComplexObject]
impl TrendingNftSchema {
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

    async fn nft(&self, ctx: &Context<'_>) -> Option<NftSchema> {
        fetch_nft(
            ctx,
            Some(self.nft_id.to_string()),
            self.collection_id.as_ref().map(|e| e.to_string()),
        )
        .await
    }
}
