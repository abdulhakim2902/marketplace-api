-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(14) UNIQUE,
    password VARCHAR(14) NOT NULL,
    role VARCHAR(10) NOT NULL,
    billing VARCHAR(15) NOT NULL,
    created_at timestamp(6) WITH time zone DEFAULT NOW() NOT NULL,
    updated_at timestamp(6) WITH time zone DEFAULT NOW() NOT NULL
);