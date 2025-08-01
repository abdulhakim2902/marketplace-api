-- Add up migration script here
CREATE TABLE IF NOT EXISTS nft_metadata (
    uri VARCHAR,
    collection_id VARCHAR(66),
    name VARCHAR DEFAULT NULL,
    description VARCHAR DEFAULT NULL,
    image VARCHAR DEFAULT NULL,
    animation_url VARCHAR DEFAULT NULL,
    avatar_url VARCHAR DEFAULT NULL,
    background_color VARCHAR DEFAULT NULL,
    image_data VARCHAR DEFAULT NULL,
    youtube_url VARCHAR DEFAULT NULL,
    external_url VARCHAR DEFAULT NULL,
    attributes JSONB DEFAULT NULL,
    properties JSONB DEFAULT NULL,
    PRIMARY KEY (uri, collection_id)
);