pub mod attribute_worker;
pub mod marketplace_processor;
pub mod price_indexer;
pub mod steps;
pub mod token_processor;

use std::{sync::Arc, time::Duration};

use tokio::time::sleep;
use tokio_util::task::TaskTracker;

use crate::{
    cache::ICache,
    config::Config,
    database::IDatabase,
    utils::shutdown_utils,
    workers::{
        attribute_worker::AttributeWorker, marketplace_processor::MarketplaceProcessor,
        price_indexer::PriceIndexer, token_processor::TokenProcessor,
    },
};

pub struct Worker<TDb: IDatabase, TCache: ICache> {
    marketplace_processor: Arc<MarketplaceProcessor<TDb, TCache>>,
    token_processor: Arc<TokenProcessor<TDb>>,
    price_indexer: Arc<PriceIndexer<TDb, TCache>>,
    attribute_worker: Arc<AttributeWorker<TDb>>,
}

impl<TDb, TCache> Worker<TDb, TCache>
where
    TDb: IDatabase + Send + Sync + 'static,
    TCache: ICache + 'static,
{
    pub fn new(config: Arc<Config>, db: Arc<TDb>, cache: Arc<TCache>) -> Self {
        Self {
            marketplace_processor: Arc::new(MarketplaceProcessor::new(
                Arc::clone(&config),
                Arc::clone(&db),
                Arc::clone(&cache),
            )),
            token_processor: Arc::new(TokenProcessor::new(Arc::clone(&config), Arc::clone(&db))),
            price_indexer: Arc::new(PriceIndexer::new(
                config.tapp_url.clone(),
                Arc::clone(&db),
                Arc::clone(&cache),
            )),
            attribute_worker: Arc::new(AttributeWorker::new(Arc::clone(&db))),
        }
    }

    pub async fn start(self: &Arc<Self>) -> anyhow::Result<()> {
        tracing::info!("Worker started");

        let tracker = TaskTracker::new();

        let mp_self = Arc::clone(self);
        tracker.spawn(async move { mp_self.marketplace_processor.start().await });
        let pi_self = Arc::clone(self);
        tracker.spawn(async move { pi_self.price_indexer.start().await });
        let tk_self = Arc::clone(self);
        tracker.spawn(async move { tk_self.token_processor.start().await });
        let attr_self = Arc::clone(self);
        tracker.spawn(async move { attr_self.attribute_worker.start().await });

        let cancel_token = shutdown_utils::get_shutdown_token();
        tokio::select! {
            _ = cancel_token.cancelled() => {
                tracing::info!("Waiting for worker tasks to finish...");
                tracker.wait().await;
                sleep(Duration::from_secs(5)).await;
                tracing::info!("All worker tasks finished");
            }
        }

        Ok(())
    }
}
