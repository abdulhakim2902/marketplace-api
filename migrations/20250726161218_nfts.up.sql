-- Add up migration script here
CREATE TABLE IF NOT EXISTS nfts (
    id UUID PRIMARY KEY NOT NULL,
    name VARCHAR(128),
    owner VARCHAR(66),
    collection_id VARCHAR(66) DEFAULT NULL,
    token_id VARCHAR(66) NOT NULL,
    properties JSONB DEFAULT NULL,
    uri VARCHAR DEFAULT NULL,
    description VARCHAR DEFAULT NULL,
    burned BOOLEAN DEFAULT false,
    royalty NUMERIC(20, 5),
    version VARCHAR(10) DEFAULT 'v2',
    updated_at timestamp(6) WITH time zone DEFAULT NOW() NOT NULL
);
