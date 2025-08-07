-- Add down migration script here
DROP TRIGGER attributes_after_insert_calculate_rarity ON nft_attributes;

DROP FUNCTION calculate_rarity;

DROP TABLE IF EXISTS nft_attributes;

DROP TABLE IF EXISTS collection_attributes;