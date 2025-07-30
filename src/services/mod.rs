use std::sync::Arc;

use crate::{
    database::Database,
    services::health::{HealthService, IHealthService},
};

pub mod health;

pub trait IInternalServices {
    type THealthService: IHealthService + Send + Sync;
}

pub struct InternalServices;
impl IInternalServices for InternalServices {
    type THealthService = HealthService<Database>;
}

pub struct Services<TInternalService: IInternalServices> {
    pub health_service: Arc<TInternalService::THealthService>,
}

impl<TInternalService: IInternalServices> Services<TInternalService> {
    pub fn new(health_service: Arc<TInternalService::THealthService>) -> Self {
        Self { health_service }
    }
}
