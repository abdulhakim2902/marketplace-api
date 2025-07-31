-- Add up migration script here
CREATE TABLE IF NOT EXISTS wallets (
  address VARCHAR(66) UNIQUE NOT NULL,
  PRIMARY KEY (address)
)