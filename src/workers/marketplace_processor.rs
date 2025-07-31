use std::sync::Arc;

use aptos_indexer_processor_sdk::{
    aptos_indexer_transaction_stream::{
        BooleanTransactionFilter, EventFilterBuilder, MoveStructTagFilterBuilder,
        TransactionRootFilterBuilder, TransactionStreamConfig,
    },
    aptos_protos::transaction::v1::transaction::TransactionType,
    builder::ProcessorBuilder,
    common_steps::{
        DEFAULT_UPDATE_PROCESSOR_STATUS_SECS, TransactionStreamStep, VersionTrackerStep,
    },
    traits::IntoRunnableStep,
};
use futures::future::join_all;

use crate::{
    cache::ICache,
    config::{Config, marketplace_config::NFTMarketplaceConfig},
    database::{IDatabase, marketplaces::IMarketplaces, processor_status::IProcessorStatus},
    utils::shutdown_utils,
    workers::steps::{
        marketplace::{
            db_writing_step::DBWritingStep, reduction_step::NFTReductionStep,
            remapping_step::RemappingStep,
        },
        processor_status_saver_step::DbProcessorStatusSaver,
    },
};

pub struct MarketplaceProcessor<TDb: IDatabase, TCache: ICache> {
    config: Arc<Config>,
    db: Arc<TDb>,
    cache: Arc<TCache>,
}

impl<TDb, TCache> MarketplaceProcessor<TDb, TCache>
where
    TDb: IDatabase + Send + Sync + 'static,
    TCache: ICache + 'static,
{
    pub fn new(config: Arc<Config>, db: Arc<TDb>, cache: Arc<TCache>) -> Self {
        Self { config, db, cache }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        self.db
            .marketplaces()
            .insert_market_places(&self.config.nft_marketplace_configs)
            .await?;

        let pool_futures = self
            .config
            .nft_marketplace_configs
            .iter()
            .map(|config| async move {
                if let Err(e) = self.stream_marketplace_event(config).await {
                    tracing::error!(
                        err = ?e,
                        marketplace = %config.name,
                        contract_addr = %config.contract_address,
                        "Error streaming and publishing events"
                    );
                }
            })
            .collect::<Vec<_>>();

        join_all(pool_futures).await;

        Ok(())
    }

    async fn stream_marketplace_event(&self, config: &NFTMarketplaceConfig) -> anyhow::Result<()> {
        let starting_version = self
            .db
            .processor_status()
            .get_starting_version(&config.name)
            .await
            .unwrap_or(config.starting_version);

        let addr = config.contract_address.clone();

        let struct_filter_builder = MoveStructTagFilterBuilder::default()
            .address(addr)
            .build()?;

        let sc_addr_filter = EventFilterBuilder::default()
            .struct_type(struct_filter_builder)
            .build()?;

        let tx_filter = TransactionRootFilterBuilder::default()
            .success(true)
            .txn_type(TransactionType::User)
            .build()?;

        let filter = BooleanTransactionFilter::from(tx_filter).and(sc_addr_filter);

        let transaction_stream = TransactionStreamStep::new(TransactionStreamConfig {
            indexer_grpc_data_service_address: url::Url::parse(
                &self.config.stream_config.indexer_grpc,
            )?,
            starting_version: Some(starting_version as u64),
            request_ending_version: Some(9805408995),
            auth_token: self.config.stream_config.auth_token.clone(),
            request_name_header: "marketplace-event-processor".to_string(),
            additional_headers: Default::default(),
            indexer_grpc_http2_ping_interval_secs: 30,
            indexer_grpc_http2_ping_timeout_secs: 10,
            indexer_grpc_reconnection_timeout_secs: 5,
            indexer_grpc_response_item_timeout_secs: 60,
            indexer_grpc_reconnection_max_retries: 5,
            transaction_filter: Some(filter),
        })
        .await?;

        let remapping_step = RemappingStep::new(config.clone())?;
        let reduction_step = NFTReductionStep::new(Arc::clone(&self.db), Arc::clone(&self.cache));
        let db_writing_step = DBWritingStep::new(Arc::clone(&self.db));
        let version_tracker_step = VersionTrackerStep::new(
            DbProcessorStatusSaver::new(config.name.clone(), Arc::clone(&self.db)),
            DEFAULT_UPDATE_PROCESSOR_STATUS_SECS,
        );

        let (_, buffer_receiver) = ProcessorBuilder::new_with_inputless_first_step(
            transaction_stream.into_runnable_step(),
        )
        .connect_to(remapping_step.into_runnable_step(), 10)
        .connect_to(reduction_step.into_runnable_step(), 10)
        .connect_to(db_writing_step.into_runnable_step(), 10)
        .connect_to(version_tracker_step.into_runnable_step(), 10)
        .end_and_return_output_receiver(10);

        let cancel_token = shutdown_utils::get_shutdown_token();
        tokio::select! {
            _ = async {
                loop {
                    if cancel_token.is_cancelled() {
                        break;
                    }

                    match buffer_receiver.recv().await {
                        Ok(txn_context) => {
                            tracing::debug!(
                                "Finished processing events from versions [{:?}, {:?}]",
                                txn_context.metadata.start_version,
                                txn_context.metadata.end_version,
                            );
                        }
                        Err(e) => {
                            tracing::info!("No more transactions in channel: {:?}", e);
                            break;
                        }
                    }
                }
            } => {},
            _ = cancel_token.cancelled() => {}
        }

        Ok(())
    }
}
