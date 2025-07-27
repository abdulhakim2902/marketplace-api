use std::sync::Arc;

use crate::{
    database::Database,
    services::{
        collection::{CollectionService, ICollectionService},
        health::{HealthService, IHealthService},
    },
};

pub mod collection;
pub mod health;

pub trait IInternalServices {
    type THealthService: IHealthService + Send + Sync;
    type TCollectionService: ICollectionService + Send + Sync;
}

pub struct InternalServices;
impl IInternalServices for InternalServices {
    type THealthService = HealthService<Database>;
    type TCollectionService = CollectionService<Database>;
}

pub struct Services<TInternalService: IInternalServices> {
    pub health_service: Arc<TInternalService::THealthService>,
    pub collection_service: Arc<TInternalService::TCollectionService>,
}

impl<TInternalService: IInternalServices> Services<TInternalService> {
    pub fn new(
        health_service: Arc<TInternalService::THealthService>,
        collection_service: Arc<TInternalService::TCollectionService>,
    ) -> Self {
        Self {
            health_service,
            collection_service,
        }
    }
}
