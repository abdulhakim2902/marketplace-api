use std::{fs::File, io::Read};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::config::marketplace_config::NFTMarketplaceConfig;

pub mod marketplace_config;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub tapp_url: String,
    pub server_config: ServerConfig,
    pub jwt_config: JWTConfig,
    pub db_config: DbConfig,
    pub stream_config: StreamConfig,
    pub nft_marketplace_configs: Vec<NFTMarketplaceConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JWTConfig {
    pub secret: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbConfig {
    pub url: String,
    #[serde(default = "DbConfig::default_db_pool_size")]
    pub pool_size: u32,
}

impl DbConfig {
    pub const fn default_db_pool_size() -> u32 {
        10
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StreamConfig {
    pub indexer_grpc: String,
    pub auth_token: String,
    pub starting_version: u64,
    pub ending_version: Option<u64>,
    pub active: bool,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let mut file = File::open("config.yaml").with_context(|| "failed to open the file path")?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .with_context(|| "failed to read the file path")?;

        let config =
            serde_yaml::from_str::<Self>(&contents).with_context(|| "failed to parse yaml file")?;

        Ok(config)
    }
}
