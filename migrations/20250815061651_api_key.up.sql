-- Add up migration script here
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE,
    name VARCHAR(14) NOT NULL,
    description VARCHAR DEFAULT NULL,
    key VARCHAR(30) NOT NULL,
    created_at timestamp(6) WITH time zone DEFAULT NOW() NOT NULL,
    updated_at timestamp(6) WITH time zone DEFAULT NOW() NOT NULL
);