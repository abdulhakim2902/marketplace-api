use std::{collections::HashMap, sync::Arc};

use futures::future::join_all;
use reqwest::Client;

use crate::{
    database::{IDatabase, attributes::IAttributes, nfts::INfts},
    models::{db::attribute::Attribute, nft_metadata::NFTMetadata},
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

    pub async fn process_attributes(&self, _client: &Client) -> anyhow::Result<()> {
        let total_nfts = self.db.nfts().count_nft_metadata_urls().await?;
        let batch_size = 20i64;
        let mut offset = 0;

        if total_nfts == 0 {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            return Ok(());
        }

        while offset < total_nfts {
            let mut nfts = self
                .db
                .nfts()
                .fetch_nft_metadata_urls(offset, batch_size)
                .await?;

            let total_nft = nfts.len() as i64;

            let nft_metadata_fut = nfts.iter().map(|nft| async move {
                let image_url = nft.image_url.as_ref().unwrap();
                let response = reqwest::get(image_url).await;
                if response.is_err() {
                    return (nft.id.clone(), None);
                }

                let value = response.unwrap().json::<NFTMetadata>().await;
                if value.is_err() {
                    return (nft.id.clone(), None);
                }

                (nft.id.clone(), Some(value.unwrap()))
            });

            let nft_metadata = join_all(nft_metadata_fut).await.into_iter().fold(
                HashMap::new(),
                |mut acc, item| {
                    let (nft_id, nft_metadata) = item;
                    if let Some(nft_metadata) = nft_metadata {
                        acc.insert(nft_id, nft_metadata);
                    }

                    acc
                },
            );

            let mut attributes = Vec::new();

            for nft in nfts.iter_mut() {
                if let Some(nft_metadata) = nft_metadata.get(&nft.id).cloned() {
                    nft.image_url = nft_metadata.image;
                    nft.youtube_url = nft_metadata.youtube_url;
                    nft.background_color = nft_metadata.background_color;
                    nft.external_url = nft_metadata.external_url;
                    nft.animation_url = nft_metadata.animation_url;
                    nft.avatar_url = nft_metadata.avatar_url;
                    nft.image_data = nft_metadata.image_data;
                    if nft.name.is_none() {
                        nft.name = nft_metadata.name;
                    }

                    if nft.description.is_none() {
                        nft.description = nft_metadata.description;
                    }

                    for attribute in nft_metadata.attributes {
                        let attribute = Attribute {
                            collection_id: nft.collection_id.clone(),
                            nft_id: Some(nft.id.clone()),
                            attr_type: Some(attribute.trait_type.to_lowercase()),
                            value: Some(attribute.value.to_lowercase()),
                        };

                        attributes.push(attribute);
                    }
                }
            }

            let mut tx = self.db.get_pool().begin().await?;

            self.db.nfts().tx_insert_nfts(&mut tx, nfts).await?;

            self.db
                .attributes()
                .tx_insert_attributes(&mut tx, attributes)
                .await?;

            tx.commit().await?;

            offset += total_nft;
        }

        // Implementation for processing attributes
        Ok(())
    }
}
