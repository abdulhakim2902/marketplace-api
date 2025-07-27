use std::{hash::Hash, time::Duration};

use bigdecimal::BigDecimal;
use moka::future::Cache as MokaCache;

#[async_trait::async_trait]
pub trait ICache: Send + Sync + 'static {
    fn is_healthy(&self) -> bool;

    async fn get_token_price(&self, token_addr: &str) -> Option<BigDecimal>;
    async fn set_token_price(&self, token_addr: &str, price: BigDecimal);
}

pub struct Cache {
    token_prices: MokaCache<String, BigDecimal>,
}

impl Cache {
    fn create_new_moka_cache<K, V>(max_capacity: u64) -> MokaCache<K, V>
    where
        K: 'static + Send + Sync + Eq + Hash,
        V: 'static + Clone + Send + Sync,
    {
        MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_idle(Duration::from_secs(3600 * 2)) // 2 hrs
            .time_to_live(Duration::from_secs(3600 * 12)) // 12 hrs
            .build()
    }

    pub fn default() -> Self {
        let token_prices = Self::create_new_moka_cache(500);

        Self { token_prices }
    }
}

#[async_trait::async_trait]
impl ICache for Cache {
    fn is_healthy(&self) -> bool {
        true
    }

    async fn get_token_price(&self, token_addr: &str) -> Option<BigDecimal> {
        self.token_prices.get(token_addr).await
    }

    async fn set_token_price(&self, token_addr: &str, price: BigDecimal) {
        self.token_prices
            .insert(token_addr.to_string(), price)
            .await;
    }
}
