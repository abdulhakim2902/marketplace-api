use crate::{
    config::marketplace_config::NFTMarketplaceConfig,
    models::marketplace::NftMarketplaceActivity,
    utils::string_utils::capitalize,
    workers::steps::marketplace::remappers::{
        event_remapper::EventRemapper, resource_remapper::ResourceMapper,
    },
};
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::Transaction,
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashMap, sync::Arc};

pub struct RemappingStep
where
    Self: Sized + Send + 'static,
{
    name: String,
    event_remapper: Arc<EventRemapper>,
    resource_remapper: Arc<ResourceMapper>,
}

impl RemappingStep {
    pub fn new(config: NFTMarketplaceConfig) -> anyhow::Result<Self> {
        let event_remapper: Arc<EventRemapper> = EventRemapper::new(&config)?;
        let resource_remapper: Arc<ResourceMapper> = ResourceMapper::new(&config)?;

        Ok(Self {
            name: config.name,
            event_remapper,
            resource_remapper,
        })
    }
}

#[async_trait::async_trait]
impl Processable for RemappingStep {
    type Input = Vec<Transaction>;
    type Output = (
        Vec<NftMarketplaceActivity>,
        HashMap<String, HashMap<String, String>>,
    );
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        transactions: TransactionContext<Vec<Transaction>>,
    ) -> Result<Option<TransactionContext<Self::Output>>, ProcessorError> {
        let results = transactions
            .data
            .par_iter()
            .map(|transaction| {
                let event_remapper = self.event_remapper.clone();
                let resource_remapper = self.resource_remapper.clone();

                let activities = event_remapper.remap_events(transaction.clone())?;
                let resource_updates = resource_remapper.remap_resources(transaction.clone())?;

                Ok((activities, resource_updates))
            })
            .collect::<anyhow::Result<Vec<_>>>()
            .map_err(|e| ProcessorError::ProcessError {
                message: format!("{e:#}"),
            })?;

        let (mut all_activities, mut all_resource_updates) = (
            Vec::new(),
            HashMap::<String, HashMap<String, String>>::new(),
        );

        for (activities, resource_updates) in results {
            all_activities.extend(activities);

            // Merge resource_updates by key
            resource_updates.into_iter().for_each(|(key, value_map)| {
                all_resource_updates
                    .entry(key)
                    .or_default()
                    .extend(value_map);
            });
        }

        Ok(Some(TransactionContext {
            data: (all_activities, all_resource_updates),
            metadata: transactions.metadata,
        }))
    }
}

impl AsyncStep for RemappingStep {}

impl NamedStep for RemappingStep {
    fn name(&self) -> String {
        format!("{}RemappingStep", capitalize(&self.name))
    }
}
