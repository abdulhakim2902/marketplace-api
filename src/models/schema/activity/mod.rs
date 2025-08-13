pub mod profit_loss;

use crate::models::{
    marketplace::APT_DECIMAL,
    schema::{collection::CollectionSchema, fetch_collection, fetch_nft, nft::NftSchema},
};
use async_graphql::{Context, Enum, InputObject};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize, FromRow)]
pub struct ActivitySchema {
    pub id: Uuid,
    pub tx_type: Option<String>,
    pub tx_index: i64,
    pub tx_id: String,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub price: Option<i64>,
    pub usd_price: Option<BigDecimal>,
    pub market_name: Option<String>,
    pub market_contract_id: Option<String>,
    pub nft_id: Option<Uuid>,
    pub collection_id: Option<Uuid>,
    pub block_time: Option<DateTime<Utc>>,
    pub block_height: Option<i64>,
    pub amount: Option<i64>,
}

#[async_graphql::Object]
impl ActivitySchema {
    async fn id(&self) -> String {
        self.id.to_string()
    }

    #[graphql(name = "type")]
    async fn tx_type(&self) -> Option<&str> {
        self.tx_type.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "tx_index")]
    async fn tx_index(&self) -> i64 {
        self.tx_index
    }

    #[graphql(name = "tx_id")]
    async fn tx_id(&self) -> &str {
        &self.tx_id
    }

    async fn sender(&self) -> Option<&str> {
        self.sender.as_ref().map(|e| e.as_str())
    }

    async fn receiver(&self) -> Option<&str> {
        self.receiver.as_ref().map(|e| e.as_str())
    }

    async fn price(&self) -> Option<String> {
        self.price
            .as_ref()
            .map(|e| (BigDecimal::from(*e) / APT_DECIMAL).to_plain_string())
    }

    #[graphql(name = "usd_price")]
    async fn usd_price(&self) -> Option<String> {
        self.usd_price.as_ref().map(|e| e.to_plain_string())
    }

    #[graphql(name = "market_name")]
    async fn market_name(&self) -> Option<&str> {
        self.market_name.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "market_contract_id")]
    async fn market_contract_id(&self) -> Option<&str> {
        self.market_contract_id.as_ref().map(|e| e.as_str())
    }

    #[graphql(name = "nft_id")]
    async fn nft_id(&self) -> Option<String> {
        self.nft_id.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "collection_id")]
    async fn collection_id(&self) -> Option<String> {
        self.collection_id.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "block_time")]
    async fn block_time(&self) -> Option<String> {
        self.block_time.as_ref().map(|e| e.to_string())
    }

    #[graphql(name = "block_height")]
    async fn block_height(&self) -> Option<i64> {
        self.block_height
    }

    #[graphql(name = "amount")]
    async fn amount(&self) -> Option<i64> {
        self.amount
    }

    async fn nft(&self, ctx: &Context<'_>) -> Option<NftSchema> {
        fetch_nft(
            ctx,
            self.nft_id.as_ref().map(|e| e.to_string()),
            self.collection_id.as_ref().map(|e| e.to_string()),
        )
        .await
    }

    async fn collection(&self, ctx: &Context<'_>) -> Option<CollectionSchema> {
        fetch_collection(ctx, self.collection_id.as_ref().map(|e| e.to_string())).await
    }
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
pub struct FilterActivitySchema {
    #[graphql(name = "where")]
    pub where_: Option<WhereActivitySchema>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct WhereActivitySchema {
    pub id: Option<String>,
    pub wallet_address: Option<String>,
    pub collection_id: Option<String>,
    pub nft_id: Option<String>,
    pub tx_types: Option<Vec<TxType>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[graphql(rename_items = "snake_case")]
pub enum TxType {
    Sales,
    Offers,
    Listings,
    Transfers,
    Mints,
}
