use std::sync::Arc;

use crate::{
    database::Database,
    services::{
        account::{AccountService, IAccountService},
        collection::{CollectionService, ICollectionService},
        health::{HealthService, IHealthService},
        nft::{INftService, NftService},
    },
};

pub mod account;
pub mod collection;
pub mod health;
pub mod nft;

pub trait IInternalServices {
    type THealthService: IHealthService + Send + Sync;
    type TCollectionService: ICollectionService + Send + Sync;
    type TNftService: INftService + Send + Sync;
    type TAccountService: IAccountService + Send + Sync;
}

pub struct InternalServices;
impl IInternalServices for InternalServices {
    type THealthService = HealthService<Database>;
    type TCollectionService = CollectionService<Database>;
    type TNftService = NftService<Database>;
    type TAccountService = AccountService<Database>;
}

pub struct Services<TInternalService: IInternalServices> {
    pub health_service: Arc<TInternalService::THealthService>,
    pub collection_service: Arc<TInternalService::TCollectionService>,
    pub nft_service: Arc<TInternalService::TNftService>,
    pub account_service: Arc<TInternalService::TAccountService>,
}

impl<TInternalService: IInternalServices> Services<TInternalService> {
    pub fn new(
        health_service: Arc<TInternalService::THealthService>,
        collection_service: Arc<TInternalService::TCollectionService>,
        nft_service: Arc<TInternalService::TNftService>,
        account_service: Arc<TInternalService::TAccountService>,
    ) -> Self {
        Self {
            health_service,
            collection_service,
            nft_service,
            account_service,
        }
    }
}
