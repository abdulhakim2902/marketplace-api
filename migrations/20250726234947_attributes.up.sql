-- Add up migration script here
CREATE TABLE IF NOT EXISTS attributes (
  collection_id VARCHAR(66) NOT NULL,
  nft_id VARCHAR(66) NOT NULL,
  attr_type VARCHAR DEFAULT NULL,
  value VARCHAR DEFAULT NULL,
  PRIMARY KEY (collection_id, nft_id, attr_type, value)
)