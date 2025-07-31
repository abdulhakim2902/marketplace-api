-- Add up migration script here
CREATE TABLE IF NOT EXISTS marketplaces (
    name VARCHAR(128) UNIQUE NOT NULL,
    contract_address VARCHAR(66) UNIQUE NOT NULL,
    PRIMARY KEY (name, contract_address)
)