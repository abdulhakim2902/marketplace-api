-- Add up migration script here
CREATE TABLE IF NOT EXISTS request_logs (
    api_key_id UUID NOT NULL,
    count BIGINT DEFAULT 0,
    ts timestamp(6) WITH time zone NOT NULL,
    PRIMARY KEY (api_key_id, ts)
);