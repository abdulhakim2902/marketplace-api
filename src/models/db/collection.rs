use crate::models::EventModel;
use crate::utils::generate_collection_id;
use crate::utils::token_utils::TokenEvent;
use crate::{
    models::resources::{
        FromWriteResource,
        collection::Collection as CollectionResourceData,
        token::{CollectionDataIdType, TokenWriteSet},
    },
    utils::{object_utils::ObjectAggregatedData, token_utils::TableMetadataForToken},
};
use ahash::AHashMap;
use anyhow::Result;
use aptos_indexer_processor_sdk::utils::extract::hash_str;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::{WriteResource, WriteTableItem},
    utils::convert::standardize_address,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DbCollection {
    pub id: Uuid,
    pub slug: Option<String>,
    pub supply: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub verified: Option<bool>,
    pub website: Option<String>,
    pub discord: Option<String>,
    pub twitter: Option<String>,
    pub royalty: Option<BigDecimal>,
    pub creator_address: Option<String>,
    pub table_handle: Option<String>,
}

impl DbCollection {
    pub async fn get_from_write_table_item(
        table_item: &WriteTableItem,
        txn_version: i64,
        table_handle_to_owner: &AHashMap<String, TableMetadataForToken>,
        conn: &Pool<Postgres>,
    ) -> Result<Option<Self>> {
        if let Some(table_item_data) = table_item.data.as_ref() {
            let maybe_collection_data = match TokenWriteSet::from_table_item_type(
                table_item_data.value_type.as_str(),
                &table_item_data.value,
                txn_version,
            )? {
                Some(TokenWriteSet::CollectionData(inner)) => Some(inner),
                _ => None,
            };

            if let Some(collection_data) = maybe_collection_data {
                let table_handle = table_item.handle.to_string();
                let maybe_creator_address =
                    match table_handle_to_owner.get(&standardize_address(&table_handle)) {
                        Some(metadata) => Some(metadata.get_owner_address()),
                        None => Self::get_collection_creator_for_v1(conn, &table_handle, 5, 500)
                            .await
                            .ok(),
                    };

                if let Some(creator_address) = maybe_creator_address {
                    let collection_id_struct = CollectionDataIdType::new(
                        creator_address.clone(),
                        collection_data.name.clone(),
                    );

                    let collection_addr = collection_id_struct.to_addr();

                    let collection = DbCollection {
                        id: generate_collection_id(collection_addr.as_str()),
                        slug: Some(get_collection_slug(
                            &creator_address,
                            collection_data.name.as_str(),
                        )),
                        title: Some(collection_data.name.clone()),
                        description: Some(collection_data.description.clone()),
                        supply: collection_data.supply.to_i64(),
                        cover_url: Some(collection_data.uri.clone()),
                        creator_address: Some(creator_address),
                        table_handle: Some(standardize_address(&table_handle)),
                        ..Default::default()
                    };

                    return Ok(Some(collection));
                }
            }
        }

        Ok(None)
    }

    pub fn get_from_write_resource(
        wr: &WriteResource,
        object_metadata: &AHashMap<String, ObjectAggregatedData>,
    ) -> Result<Option<Self>> {
        if let Some(inner) = CollectionResourceData::from_write_resource(wr)? {
            let address = standardize_address(&wr.address);

            let mut collection = DbCollection {
                id: generate_collection_id(address.as_str()),
                creator_address: Some(inner.get_creator_address()),
                slug: Some(address.clone()),
                title: Some(inner.name),
                description: Some(inner.description),
                cover_url: Some(inner.uri),
                ..Default::default()
            };

            if let Some(object) = object_metadata.get(&address) {
                if let Some(fixed_supply) = object.fixed_supply.as_ref() {
                    collection.supply = fixed_supply.current_supply.to_i64();
                }

                if let Some(unlimited_supply) = object.unlimited_supply.as_ref() {
                    collection.supply = unlimited_supply.current_supply.to_i64()
                }

                if let Some(concurrent_supply) = object.concurrent_supply.as_ref() {
                    collection.supply = concurrent_supply.current_supply.value.to_i64()
                }

                if let Some(royalty) = object.royalty.as_ref() {
                    collection.royalty = Some(royalty.get_royalty());
                }
            }

            return Ok(Some(collection));
        }

        Ok(None)
    }

    pub fn get_from_create_token_event(
        event: &EventModel,
        txn_version: i64,
    ) -> Result<Option<Self>> {
        if let Some(token) =
            TokenEvent::from_event(&event.type_str, &event.data.to_string(), txn_version)?
        {
            if let TokenEvent::CreateTokenDataEvent(inner) = token {
                let creator_address = standardize_address(&inner.id.get_creator_address());
                let collection = inner.id.collection;
                let input = format!("{}::{}", &creator_address, &collection);
                let hash_str = hash_str(&input);

                let collection = DbCollection {
                    id: generate_collection_id(&standardize_address(hash_str.as_str())),
                    slug: Some(get_collection_slug(&creator_address, &collection)),
                    title: Some(collection),
                    ..Default::default()
                };

                return Ok(Some(collection));
            }
        }

        Ok(None)
    }

    async fn get_collection_creator_for_v1(
        conn: &Pool<Postgres>,
        table_handle: &str,
        query_retries: u32,
        query_retry_delay_ms: u64,
    ) -> anyhow::Result<String> {
        let mut tried = 0;

        while tried < query_retries {
            tried += 1;

            let res = sqlx::query!(
                r#"
                SELECT creator_address FROM collections
                WHERE table_handle = $1
                "#,
                table_handle
            )
            .fetch_one(&*conn)
            .await;

            match res {
                Ok(res) => {
                    if let Some(creator_address) = res.creator_address {
                        return Ok(creator_address)
                    }
                }
                Err(_) => {
                    if tried < query_retries {
                        tokio::time::sleep(std::time::Duration::from_millis(query_retry_delay_ms))
                            .await;
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Failed to get collection creator"))
    }
}

fn get_collection_slug(creator: &str, collection: &str) -> String {
    let collection = collection
        .chars()
        .filter(|&c| c.is_alphanumeric() || c == ' ')
        .collect::<String>();
    let name = collection
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-");

    let split_addr = creator.split("").collect::<Vec<&str>>();
    let trunc_addr = &split_addr[3..11].join("");

    format!("{}-{}", name, trunc_addr).to_lowercase()
}
