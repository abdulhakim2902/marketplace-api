-- Add up migration script here
CREATE TABLE IF NOT EXISTS request_logs (
    id UUID PRIMARY KEY NOT NULL,
    api_key_id UUID NOT NULL,
    count BIGINT DEFAULT 0,
    ts timestamp(6) WITH time zone NOT NULL,
    FOREIGN KEY (api_key_id) REFERENCES api_keys(id) ON DELETE CASCADE
);