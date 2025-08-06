-- Add up migration script here
CREATE TABLE IF NOT EXISTS listings (
  id VARCHAR(66) NOT NULL,
  block_height BIGINT,
  block_time timestamp(6) WITH time zone DEFAULT NOW() NOT NULL,
  market_contract_id VARCHAR(66) DEFAULT NULL,
  collection_id VARCHAR(66),
  nft_id VARCHAR(66) NOT NULL,
  listed BOOLEAN DEFAULT NULL,
  market_name VARCHAR(128),
  nonce VARCHAR(128) DEFAULT NULL,
  price BIGINT DEFAULT NULL,
  seller VARCHAR(66),
  tx_index BIGINT,
  PRIMARY KEY (id)
)
