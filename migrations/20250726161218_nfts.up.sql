-- Add up migration script here
CREATE TABLE IF NOT EXISTS nfts (
    id UUID PRIMARY KEY NOT NULL,
    name VARCHAR(128),
    owner VARCHAR(66),
    collection_id UUID DEFAULT NULL,
    token_id VARCHAR(66) DEFAULT NULL,
    properties JSONB DEFAULT NULL,
    uri VARCHAR DEFAULT NULL,
    description VARCHAR DEFAULT NULL,
    burned BOOLEAN DEFAULT false,
    royalty NUMERIC(20, 5),
    rarity NUMERIC(20, 10) DEFAULT 0,
    version VARCHAR(10) DEFAULT 'v2',
    media_url VARCHAR DEFAULT NULL,
    animation_url VARCHAR DEFAULT NULL,
    avatar_url VARCHAR DEFAULT NULL,
    youtube_url VARCHAR DEFAULT NULL,
    external_url VARCHAR DEFAULT NULL,
    background_color VARCHAR DEFAULT NULL,
    updated_at timestamp(6) WITH time zone DEFAULT NOW() NOT NULL
);
