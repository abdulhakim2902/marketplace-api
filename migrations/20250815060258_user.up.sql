-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY NOT NULL,
    username VARCHAR(20) NOT NULL,
    password VARCHAR NOT NULL,
    role VARCHAR(10) NOT NULL,
    billing VARCHAR(15),
    active boolean DEFAULT TRUE NOT NULL,
    created_at timestamp(6) WITH time zone DEFAULT NOW() NOT NULL,
    updated_at timestamp(6) WITH time zone DEFAULT NOW() NOT NULL
);