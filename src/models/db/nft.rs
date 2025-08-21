use crate::models::db::collection::{DbCollection, get_collection_slug};
use crate::utils::{generate_collection_id, generate_nft_id};
use crate::{
    models::resources::{
        FromWriteResource, TYPE_TOKEN_STORE_V1,
        token::{Token as TokenResourceData, TokenWriteSet},
    },
    utils::{object_utils::ObjectAggregatedData, token_utils::TableMetadataForToken},
};
use ahash::{AHashMap, HashMap};
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::{WriteResource, WriteTableItem},
    utils::convert::standardize_address,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DbNft {
    pub id: Uuid,
    pub name: Option<String>,
    pub owner: Option<String>,
    pub collection_id: Option<Uuid>,
    pub burned: Option<bool>,
    pub properties: Option<serde_json::Value>,
    pub description: Option<String>,
    pub token_id: Option<String>,
    pub uri: Option<String>,
    pub royalty: Option<BigDecimal>,
    pub version: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    // FROM NFT METADATA
    pub media_url: Option<String>,
    pub animation_url: Option<String>,
    pub avatar_url: Option<String>,
    pub youtube_url: Option<String>,
    pub external_url: Option<String>,
    pub background_color: Option<String>,
}

impl DbNft {
    pub fn get_from_write_resource(
        wr: &WriteResource,
        object_metadata: &AHashMap<String, ObjectAggregatedData>,
    ) -> Result<Option<Self>> {
        if let Some(inner) = TokenResourceData::from_write_resource(wr)? {
            let token_addr = standardize_address(&wr.address);

            let mut nft = DbNft {
                id: generate_nft_id(token_addr.as_str()),
                collection_id: Some(generate_collection_id(
                    inner.get_collection_address().as_str(),
                )),
                name: Some(inner.name),
                uri: Some(inner.uri.clone()),
                description: Some(inner.description),
                token_id: Some(token_addr.clone()),
                version: Some("v2".to_string()),
                ..Default::default()
            };

            if let Some(object_data) = object_metadata.get(&token_addr) {
                let object_core = object_data.object.object_core.clone();
                let owner_address = object_core.get_owner_address();

                nft.owner = Some(owner_address);

                if let Some(token_identifier) = object_data.token_identifiers.as_ref() {
                    nft.name = Some(token_identifier.name.value.clone());
                }

                if let Some(property_map) = object_data.property_map.as_ref() {
                    nft.properties = Some(property_map.inner.clone());
                }

                if let Some(royalty) = object_data.royalty.as_ref() {
                    nft.royalty = Some(royalty.get_royalty());
                }
            }

            return Ok(Some(nft));
        }

        Ok(None)
    }

    pub fn get_from_write_table_item(
        table_item: &WriteTableItem,
        txn_version: i64,
        table_handle_to_owner: &AHashMap<String, TableMetadataForToken>,
        deposit_event_owner: &HashMap<Uuid, Option<String>>,
    ) -> Result<(Option<Self>, Option<DbCollection>)> {
        if let Some(table_item_data) = table_item.data.as_ref() {
            let maybe_token_data = match TokenWriteSet::from_table_item_type(
                &table_item_data.value_type,
                &table_item_data.value,
                txn_version,
            )? {
                Some(TokenWriteSet::TokenData(inner)) => Some(inner),
                _ => None,
            };

            if let Some(token_data) = maybe_token_data {
                let table_handle = standardize_address(&table_item.handle.to_string());
                let maybe_token_data_id = match TokenWriteSet::from_table_item_type(
                    &table_item_data.key_type,
                    &table_item_data.key,
                    txn_version,
                )? {
                    Some(TokenWriteSet::TokenDataId(inner)) => Some(inner),
                    _ => None,
                };

                if let Some(token_data_id_struct) = maybe_token_data_id {
                    let nft_id = generate_nft_id(token_data_id_struct.to_addr().as_str());
                    let owner_address = match table_handle_to_owner.get(&table_handle) {
                        Some(tm) if tm.table_type == TYPE_TOKEN_STORE_V1 => {
                            Some(tm.get_owner_address())
                        }
                        _ => {
                            let deposit_event_owner = deposit_event_owner.get(&nft_id).cloned();

                            if deposit_event_owner.is_none() {
                                None
                            } else {
                                deposit_event_owner.unwrap()
                            }
                        }
                    };

                    let collection = DbCollection {
                        id: generate_collection_id(&token_data_id_struct.get_collection_addr()),
                        creator_address: Some(token_data_id_struct.get_creator_address()),
                        slug: Some(get_collection_slug(
                            &token_data_id_struct.get_creator_address(),
                            &token_data_id_struct.collection,
                        )),
                        title: Some(token_data_id_struct.collection.clone()),
                        ..Default::default()
                    };

                    let nft = DbNft {
                        id: nft_id,
                        owner: owner_address,
                        collection_id: Some(generate_collection_id(
                            token_data_id_struct.get_collection_addr().as_str(),
                        )),
                        name: Some(token_data.name.clone()),
                        uri: Some(token_data.uri.clone()),
                        token_id: Some(token_data.name.replace(" ", "%20")),
                        properties: Some(token_data.default_properties),
                        description: Some(token_data.description),
                        royalty: Some(token_data.royalty.get_royalty()),
                        version: Some("v1".to_string()),
                        ..Default::default()
                    };

                    return Ok((Some(nft), Some(collection)));
                }
            }
        }

        Ok((None, None))
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DbNftUri {
    pub collection_id: Option<Uuid>,
    pub uri: Option<String>,
    pub nft_ids: serde_json::Value,
    pub updated_at: Option<DateTime<Utc>>,
}
