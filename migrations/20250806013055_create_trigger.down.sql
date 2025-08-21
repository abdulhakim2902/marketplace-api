-- Add down migration script here
DROP TRIGGER bid_before_insert_add_collection_id ON bids;

DROP TRIGGER activity_before_insert_update_collection_sales ON activities;

DROP TRIGGER listings_after_insert_update_collection_listings ON listings;

DROP FUNCTION add_collection_id;

DROP FUNCTION update_collection_sales;

DROP FUNCTION update_collection_listings;
