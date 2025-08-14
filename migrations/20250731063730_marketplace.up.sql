-- Add up migration script here
CREATE TABLE IF NOT EXISTS marketplaces (
    id UUID PRIMARY KEY NOT NULL,
    name VARCHAR(128) NOT NULL,
    contract_address VARCHAR(66) NOT NULL
)