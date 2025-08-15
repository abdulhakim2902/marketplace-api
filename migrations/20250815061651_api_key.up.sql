-- Add up migration script here
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE NOT NULL,
    name VARCHAR(14) NOT NULL,
    description VARCHAR DEFAULT NULL,
    short_token VARCHAR(30) UNIQUE NOT NULL,
    long_token_hash VARCHAR(66) UNIQUE NOT NULL,
    created_at timestamp(6) WITH time zone DEFAULT NOW() NOT NULL,
    updated_at timestamp(6) WITH time zone DEFAULT NOW() NOT NULL
);