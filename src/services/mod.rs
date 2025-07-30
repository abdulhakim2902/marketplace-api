use std::sync::Arc;

use crate::{
    database::Database,
    services::{
        health::{HealthService, IHealthService},
        nft::{INftService, NftService},
    },
};

pub mod health;
pub mod nft;

pub trait IInternalServices {
    type THealthService: IHealthService + Send + Sync;
    type TNftService: INftService + Send + Sync;
}

pub struct InternalServices;
impl IInternalServices for InternalServices {
    type THealthService = HealthService<Database>;
    type TNftService = NftService<Database>;
}

pub struct Services<TInternalService: IInternalServices> {
    pub health_service: Arc<TInternalService::THealthService>,
    pub nft_service: Arc<TInternalService::TNftService>,
}

impl<TInternalService: IInternalServices> Services<TInternalService> {
    pub fn new(
        health_service: Arc<TInternalService::THealthService>,
        nft_service: Arc<TInternalService::TNftService>,
    ) -> Self {
        Self {
            health_service,
            nft_service,
        }
    }
}
