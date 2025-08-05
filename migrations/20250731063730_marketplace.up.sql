-- Add up migration script here
CREATE TABLE IF NOT EXISTS marketplaces (
    name VARCHAR(128) NOT NULL,
    contract_address VARCHAR(66) NOT NULL,
    PRIMARY KEY (name, contract_address)
)