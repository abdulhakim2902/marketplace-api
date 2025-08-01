use serde::{Deserialize, Serialize};

use crate::models::nft_metadata::NFTMetadata;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DbNFTMetadata {
    pub uri: Option<String>,
    pub collection_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub animation_url: Option<String>,
    pub avatar_url: Option<String>,
    pub background_color: Option<String>,
    pub image_data: Option<String>,
    pub youtube_url: Option<String>,
    pub external_url: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub properties: Option<serde_json::Value>,
}

impl From<NFTMetadata> for DbNFTMetadata {
    fn from(value: NFTMetadata) -> Self {
        Self {
            name: value.name,
            description: value.description,
            image: value.image,
            animation_url: value.animation_url,
            avatar_url: value.avatar_url,
            background_color: value.background_color,
            image_data: value.image_data,
            youtube_url: value.youtube_url,
            external_url: value.external_url,
            attributes: value.attributes,
            properties: value.properties,
            ..Default::default()
        }
    }
}
