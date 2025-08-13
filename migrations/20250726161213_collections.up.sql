-- Add up migration script here
CREATE TABLE IF NOT EXISTS collections (
    id UUID PRIMARY KEY NOT NULL,
    slug VARCHAR(66) UNIQUE,
    supply BIGINT DEFAULT 0,
    title VARCHAR(128),
    twitter VARCHAR,
    verified BOOLEAN DEFAULT false,
    website VARCHAR,
    discord VARCHAR,
    description TEXT,
    cover_url VARCHAR(512),
    royalty NUMERIC(20, 5),
    floor BIGINT DEFAULT NULL,
    listed BIGINT DEFAULT 0,
    volume BIGINT DEFAULT 0,
    volume_usd NUMERIC(20, 2) DEFAULT 0,
    sales BIGINT DEFAULT 0,
    owners BIGINT DEFAULT 0
);
