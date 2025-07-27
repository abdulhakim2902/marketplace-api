-- Add up migration script here
CREATE TABLE IF NOT EXISTS token_prices (
    token_address VARCHAR(66) NOT NULL,
    price NUMERIC(20, 2) NOT NULL,
    created_at TIMESTAMP(6) WITH time zone DEFAULT NOW(),
    PRIMARY KEY (token_address, created_at)
);