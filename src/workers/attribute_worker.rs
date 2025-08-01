use std::sync::Arc;

use futures::future::join_all;
use reqwest::Client;

use crate::{
    database::{IDatabase, attributes::IAttributes, nft_metadata::INFTMetadata, nfts::INfts},
    models::{
        db::{attribute::DbAttribute, nft_metadata::DbNFTMetadata},
        nft_metadata::{NFTMetadata, NFTMetadataAttribute},
    },
    utils::shutdown_utils,
};

pub struct AttributeWorker<TDb: IDatabase> {
    db: Arc<TDb>,
}

impl<TDb: IDatabase> AttributeWorker<TDb>
where
    TDb: IDatabase + Send + Sync + 'static,
{
    pub fn new(db: Arc<TDb>) -> Self {
        Self { db }
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

                    if let Err(e) = self.process_attributes(&client).await {
                        tracing::error!("Failed to process attributes: {e:#}");
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                }
            } => {},
            _ = cancel_token.cancelled() => {}
        }

        Ok(())
    }

    pub async fn process_attributes(&self, client: &Client) -> anyhow::Result<()> {
        loop {
            let batch_size = 20;
            let nfts = self
                .db
                .nfts()
                .fetch_nft_uri(0, batch_size)
                .await?;

            if nfts.len() <= 0 {
                break;
            }

            let nft_metadata_fut = nfts.iter().map(|nft| async move {
                let uri = nft.uri.as_ref().unwrap();
                let response = client.get(uri).send().await?;
                let result = response.json::<NFTMetadata>().await?;

                let mut nft_metadata: DbNFTMetadata = result.into();

                nft_metadata.uri = Some(uri.to_string());
                nft_metadata.collection_id = nft.collection_id.clone();

                let nft_ids = serde_json::from_value::<Vec<String>>(nft.nft_ids.clone())?;

                Ok::<(DbNFTMetadata, Vec<String>), anyhow::Error>((nft_metadata, nft_ids))
            });

            let nft_metadata_vec = join_all(nft_metadata_fut)
                .await
                .into_iter()
                .filter_map(Result::ok)
                .collect::<Vec<(DbNFTMetadata, Vec<String>)>>();

            let mut all_attributes = Vec::new();
            let mut all_nft_metadata = Vec::new();

            for (nft_metadata, nft_ids) in nft_metadata_vec.iter() {
                let nft_attributes = nft_metadata.attributes.as_ref().map(|e| {
                    serde_json::from_value::<Vec<NFTMetadataAttribute>>(e.clone())
                        .unwrap_or_default()
                });

                for nft_id in nft_ids {
                    if let Some(nft_attributes) = nft_attributes.as_ref() {
                        for attribute in nft_attributes {
                            let nft_attribute = DbAttribute {
                                collection_id: nft_metadata.collection_id.clone(),
                                nft_id: Some(nft_id.to_string()),
                                attr_type: Some(attribute.trait_type.to_lowercase()),
                                value: Some(attribute.value.to_lowercase()),
                            };

                            all_attributes.push(nft_attribute);
                        }
                    }
                }

                all_nft_metadata.push(nft_metadata.clone());
            }

            let mut tx = self.db.get_pool().begin().await?;

            self.db
                .nft_metadata()
                .tx_insert_nft_metadata(&mut tx, all_nft_metadata)
                .await?;

            self.db
                .attributes()
                .tx_insert_attributes(&mut tx, all_attributes)
                .await?;

            tx.commit().await?;
        }

        Ok(())
    }
}
