use crate::{
    cache::ICache,
    database::{IDatabase, token_prices::ITokenPrices},
    models::db::token_price::DbTokenPrice,
    utils::shutdown_utils,
};
use aptos_indexer_processor_sdk::utils::convert::deserialize_from_string;
use bigdecimal::BigDecimal;
use chrono::{Datelike, TimeZone, Timelike, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    base_currency: String,
    name: String,
    price_decimals: i32,
    quote_currency: String,
    #[serde(deserialize_with = "deserialize_from_string")]
    price: BigDecimal,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceResponse {
    id: i32,
    jsonrpc: String,
    method: String,
    result: Price,
}

pub struct PriceIndexer<TDb: IDatabase, TCache: ICache> {
    tapp_url: String,
    db: Arc<TDb>,
    cache: Arc<TCache>,
}

impl<TDb: IDatabase, TCache: ICache> PriceIndexer<TDb, TCache>
where
    TDb: IDatabase + Send + Sync + 'static,
    TCache: ICache + 'static,
{
    pub fn new(tapp_url: String, db: Arc<TDb>, cache: Arc<TCache>) -> Self {
        Self {
            tapp_url,
            db,
            cache,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let client = Client::new();

        let cancel_token = shutdown_utils::get_shutdown_token();
        tokio::select! {
            _ = async {
                loop {
                    if cancel_token.is_cancelled() {
                        break;
                    }

                    if let Err(e) = self.fetch_and_store_token_prices(&client).await {
                        tracing::error!("Failed to fetch and store prices: {e:#}");
                    }

                    sleep(Duration::from_secs(5 * 60)).await;
                }
            } => {},
            _ = cancel_token.cancelled() => {}
        }

        Ok(())
    }

    pub async fn fetch_and_store_token_prices(&self, client: &Client) -> anyhow::Result<()> {
        let now = Utc::now();
        let rounded = Utc
            .with_ymd_and_hms(
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                0,
            )
            .unwrap();

        let body = serde_json::json!({
            "method": "public/get_index_price",
            "jsonrpc": "2.0",
            "id": now.timestamp(),
            "params": {
                "name": "0x000000000000000000000000000000000000000000000000000000000000000a_usd"
            }
        });

        let value = client
            .post(&self.tapp_url)
            .json(&body)
            .send()
            .await?
            .json::<PriceResponse>()
            .await?;

        self.cache
            .set_token_price(&value.result.base_currency, value.result.price.clone())
            .await;
        self.db
            .token_prices()
            .insert_token_price(&DbTokenPrice {
                token_address: value.result.base_currency,
                price: value.result.price,
                created_at: rounded,
            })
            .await?;

        Ok(())
    }
}
