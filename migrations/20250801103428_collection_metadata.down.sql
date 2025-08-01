-- Add down migration script here
DROP TABLE IF EXISTS collection_metadata;

DROP TRIGGER activities_after_insert_update_collection_sales ON activities;

DROP FUNCTION update_collection_sales;

DROP TRIGGER listings_after_insert_update_collection_listings ON listings;

DROP FUNCTION update_collection_listings;

DROP TRIGGER nfts_after_insert_update_collection_owners ON nfts;

DROP FUNCTION update_collection_owners;
