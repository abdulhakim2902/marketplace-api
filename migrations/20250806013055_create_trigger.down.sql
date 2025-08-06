-- Add down migration script here
DROP TRIGGER bid_before_insert_add_collection_id ON bids;

DROP TRIGGER activity_before_insert_add_collection_id ON activities;

DROP FUNCTION add_collection_id;
