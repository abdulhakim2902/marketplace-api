-- Add up migration script here
CREATE TABLE IF NOT EXISTS processor_status (
    processor VARCHAR(30) NOT NULL,
    last_success_version BIGINT NOT NULL,
    last_transaction_timestamp TIMESTAMP(6) WITH time zone DEFAULT NOW(),
    PRIMARY KEY (processor)
);