use std::sync::Arc;

use crate::models::{
    db::nft::{DbNft, DbNftUri},
    schema::nft::{CoinType, FilterType, NftSchema, OrderNftSchema, WhereNftSchema},
};
use anyhow::Context;
use chrono::Utc;
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, postgres::PgQueryResult};

#[async_trait::async_trait]
pub trait INfts: Send + Sync {
    async fn tx_insert_nfts(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbNft>,
    ) -> anyhow::Result<PgQueryResult>;

    async fn fetch_nfts(
        &self,
        query: &WhereNftSchema,
        order: Option<OrderNftSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<NftSchema>>;

    async fn fetch_nft_uri(&self, offset: i64, limit: i64) -> anyhow::Result<Vec<DbNftUri>>;

    async fn fetch_total_nft(
        &self,
        wallet_address: &str,
        collection_id: &str,
    ) -> anyhow::Result<i64>;
}

pub struct Nfts {
    pool: Arc<PgPool>,
}

impl Nfts {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl INfts for Nfts {
    async fn tx_insert_nfts(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        items: Vec<DbNft>,
    ) -> anyhow::Result<PgQueryResult> {
        if items.is_empty() {
            return Ok(PgQueryResult::default());
        }

        let res = QueryBuilder::<Postgres>::new(
            r#"
            INSERT INTO nfts (
                id,
                name,
                owner,
                collection_id,
                properties,
                image_url,
                description,
                burned,
                version,
                royalty,
                updated_at,
                uri
            )
            "#,
        )
        .push_values(items, |mut b, item| {
            b.push_bind(item.id.clone());
            b.push_bind(item.name.clone());
            b.push_bind(item.owner.clone());
            b.push_bind(item.collection_id.clone());
            b.push_bind(item.properties.clone());
            b.push_bind(item.image_url.clone());
            b.push_bind(item.description.clone());
            b.push_bind(item.burned);
            b.push_bind(item.version.clone());
            b.push_bind(item.royalty.clone());
            b.push_bind(Utc::now());
            b.push_bind(item.uri);
        })
        .push(
            r#"
            ON CONFLICT (id) DO UPDATE SET
                owner = EXCLUDED.owner,
                name = COALESCE(EXCLUDED.name, nfts.name),
                uri = COALESCE(EXCLUDED.uri, nfts.uri),
                image_url = COALESCE(EXCLUDED.image_url, nfts.image_url),
                description = COALESCE(EXCLUDED.description, nfts.description),
                properties = COALESCE(EXCLUDED.properties, nfts.properties),
                royalty = COALESCE(EXCLUDED.royalty, nfts.royalty),
                burned = EXCLUDED.burned,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .build()
        .execute(&mut **tx)
        .await
        .context("Failed to insert nfts")?;

        Ok(res)
    }

    async fn fetch_nfts(
        &self,
        query: &WhereNftSchema,
        order: Option<OrderNftSchema>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<NftSchema>> {
        let mut query_builder = QueryBuilder::<Postgres>::new(
            r#"
            WITH
                latest_prices AS (
                    SELECT DISTINCT ON (tp.token_address) tp.token_address, tp.price FROM token_prices tp
                    WHERE tp.token_address = '0x000000000000000000000000000000000000000000000000000000000000000a'
                    ORDER BY tp.token_address, tp.created_at DESC
                ),
                listing_prices AS (
                    SELECT 
                        l.nft_id,
                        l.price,
                        l.block_time,
                        l.seller,
                        l.market_name,
                        l.market_contract_id,
                        l.nonce
                    FROM listings l
                    WHERE l.listed
                ),
                bid_prices AS (
                    SELECT DISTINCT ON (b.nft_id)
                        b.bidder,
                        b.nft_id,
                        b.price,
                        b.nonce
                    FROM bids b
                    WHERE b.status = 'active' 
                        AND b.expired_at > NOW()
                        AND b.bid_type = 'solo'
                ),
                sales AS (
                    SELECT DISTINCT ON (a.nft_id) 
                        a.nft_id, 
                        a.block_time, 
                        a.price 
                    FROM activities a
                    WHERE a.tx_type = 'buy'
                    ORDER BY a.nft_id, a.block_time DESC
                ),
                attribute_rarities AS (
                    SELECT
                        a.collection_id,
                        a.nft_id,
                        SUM(ar.score)            AS score
                    FROM attributes a
                        LEFT JOIN attribute_rarities ar ON a.collection_id = ar.collection_id
                                                                AND a.attr_type = ar.type
                                                                AND a.value = ar.value
                    GROUP BY a.collection_id, a.nft_id
                ),
                nfts AS (
                    SELECT 
                        id,
                        COALESCE(n.name, nm.name)                   AS name,
                        owner,
                        n.collection_id,
                        burned,
                        n.properties,
                        COALESCE(n.description, nm.description)     AS description,
                        COALESCE(nm.image, n.uri)                   AS image_url,
                        nm.animation_url,
                        nm.avatar_url,
                        nm.youtube_url,
                        nm.external_url,
                        nm.background_color,
                        royalty,
                        version,
                        updated_at,
                        lp.price                                    AS list_price,
                        lp.price * ltp.price                        AS list_usd_price,
                        lp.market_name,
                        lp.market_contract_id,
                        lp.nonce                                    AS list_nonce,
                        bp.nonce                                    AS bid_nonce,
                        bpa.nonce                                   AS auction_nonce,
                        CASE
                        WHEN lp.block_time IS NOT NULL
                            THEN lp.block_time
                            ELSE NULL
                        END                                         AS listed_at,
                        s.price                                     AS last_sale,
                        s.block_time                                AS received_at,
                        ar.score,
                        CASE
                            WHEN ar.score IS NOT NULL
                            THEN RANK () OVER (
                                PARTITION BY n.collection_id
                                ORDER BY ar.score DESC
                            )
                            END                                     AS rank
                    FROM nfts n
                        LEFT JOIN nft_metadata nm ON nm.uri = n.uri AND nm.collection_id = n.collection_id
                        LEFT JOIN attribute_rarities ar ON ar.nft_id = n.id AND ar.collection_id = n.collection_id
                        LEFT JOIN listing_prices lp ON lp.nft_id = n.id AND lp.seller = n.owner
                        LEFT JOIN sales s ON s.nft_id = n.id
                        LEFT JOIN bid_prices bp ON bp.nft_id = n.id AND bp.bidder != n.owner
                        LEFT JOIN bid_prices bpa ON bpa.nft_id = n.id AND bpa.bidder = n.owner
                        LEFT JOIN latest_prices ltp ON TRUE
                )
            SELECT
                n.id,
                n.name,
                n.owner,
                n.collection_id,
                n.burned,
                n.properties,
                n.description,
                n.image_url,
                n.royalty,
                n.version,
                n.updated_at,
                n.list_price,
                n.list_usd_price,
                n.listed_at,
                n.last_sale,
                n.score,
                n.rank,
                n.animation_url,
                n.avatar_url,
                n.youtube_url,
                n.external_url,
                n.background_color
            FROM nfts n
            WHERE TRUE
            "#,
        );

        if let Some(type_) = query.type_.as_ref() {
            match type_ {
                FilterType::Listed => {
                    query_builder.push(" AND n.list_nonce IS NOT NULL ");
                }
                FilterType::HasOffer => {
                    query_builder.push(" AND n.bid_nonce IS NOT NULL ");
                }
                FilterType::OnAuction => {
                    query_builder.push(" AND n.auction_nonce IS NOT NULL ");
                }
                _ => {}
            }
        }

        if let Some(search) = query.search.as_ref() {
            query_builder.push(" AND n.name ILIKE ");
            query_builder.push_bind(search);
        }

        if let Some(nft_id) = query.nft_id.as_ref() {
            query_builder.push(" AND n.id = ");
            query_builder.push_bind(nft_id);
        }

        if let Some(collection_id) = query.collection_id.as_ref() {
            query_builder.push(" AND n.collection_id = ");
            query_builder.push_bind(collection_id);
        }

        if let Some(wallet_address) = query.wallet_address.as_ref() {
            query_builder.push(" AND n.owner = ");
            query_builder.push_bind(wallet_address);
        }

        if let Some(burned) = query.burned {
            if burned {
                query_builder.push(" AND n.burned");
            } else {
                query_builder.push(" AND NOT n.burned");
            }
        }

        if let Some(rank) = query.rarity.as_ref() {
            query_builder.push(" AND n.rank >= ");
            query_builder.push(rank.min);

            if let Some(max) = rank.max {
                query_builder.push(" AND n.rank <= ");
                query_builder.push(max);
            }
        }

        if let Some(contract_ids) = query.market_contract_ids.as_ref() {
            query_builder.push(" AND n.market_contract_id = ANY(");
            query_builder.push_bind(contract_ids);
            query_builder.push(")");
        }

        if let Some(price) = query.price.as_ref() {
            match price.type_ {
                CoinType::APT => {
                    query_builder.push(" AND n.list_price >= ");
                    query_builder.push_bind(&price.range.min);

                    if let Some(max) = price.range.max.as_ref() {
                        query_builder.push(" AND n.list_price <= ");
                        query_builder.push_bind(max);
                    }
                }
                CoinType::USD => {
                    query_builder.push(" AND n.list_usd_price >= ");
                    query_builder.push_bind(&price.range.min);

                    if let Some(max) = price.range.max.as_ref() {
                        query_builder.push(" AND n.list_usd_price <= ");
                        query_builder.push_bind(max);
                    }
                }
            }
        }

        if let Some(attributes) = query.attributes.as_ref() {
            for attribute in attributes {
                query_builder.push(" AND n.id IN (SELECT a.nft_id FROM attributes a WHERE TRUE");

                if let Some(collection_id) = query.collection_id.as_ref() {
                    query_builder.push(" AND a.collection_id = ");
                    query_builder.push_bind(collection_id);
                }

                query_builder.push(" AND a.attr_type = ");
                query_builder.push_bind(attribute.type_.as_str());
                query_builder.push(" AND a.value = ANY(");
                query_builder.push_bind(attribute.values.as_slice());
                query_builder.push("))");
            }
        }

        if let Some(order) = order.as_ref() {
            let mut order_builder = String::new();
            if let Some(order_type) = order.price {
                order_builder
                    .push_str(format!(" n.list_price {},", order_type.to_string()).as_str());
            }

            if let Some(order_type) = order.rarity {
                order_builder.push_str(format!(" n.score {},", order_type.to_string()).as_str());
            }

            if let Some(order_type) = order.listed_at {
                order_builder
                    .push_str(format!(" n.listed_at {},", order_type.to_string()).as_str());
            }

            if let Some(order_type) = order.received_at {
                order_builder
                    .push_str(format!(" n.received_at {},", order_type.to_string()).as_str());
            }

            let ordering = &order_builder[..(order_builder.len() - 1)];
            if ordering.trim().is_empty() {
                query_builder.push(" ORDER BY n.list_price DESC ");
            } else {
                query_builder.push(format!(" ORDER BY {}", ordering.to_lowercase().trim()));
            }
        }

        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let res = query_builder
            .build_query_as::<NftSchema>()
            .fetch_all(&*self.pool)
            .await
            .context("Failed to fetch nfts")?;

        Ok(res)
    }

    async fn fetch_nft_uri(&self, offset: i64, limit: i64) -> anyhow::Result<Vec<DbNftUri>> {
        let res = sqlx::query_as!(
            DbNftUri,
            r#"
            SELECT 
                n.collection_id, 
                n.uri, 
                jsonb_agg(DISTINCT n.id)    AS nft_ids,
                MIN(n.updated_at)           AS updated_at              
            FROM nfts n
                LEFT JOIN nft_metadata nm ON nm.uri = n.uri 
            WHERE n.uri ILIKE '%.json' AND nm.uri IS NULL
            GROUP BY n.collection_id, n.uri
            ORDER BY updated_at ASC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch nft metadata urls")?;

        Ok(res)
    }

    async fn fetch_total_nft(
        &self,
        wallet_address: &str,
        collection_id: &str,
    ) -> anyhow::Result<i64> {
        let res = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM nfts n
            WHERE n.owner = $1 AND n.collection_id = $2
            "#,
            wallet_address,
            collection_id,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to count total nft")?;

        Ok(res.unwrap_or_default())
    }
}
