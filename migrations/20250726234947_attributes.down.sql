-- Add down migration script here
DROP TRIGGER IF EXISTS attributes_before_insert_rarity_and_score ON attributes;

DROP TRIGGER IF EXISTS attributes_after_update_nft_score ON attributes;

DROP FUNCTION IF EXISTS update_rarity_and_score;

DROP FUNCTION IF EXISTS update_nft_score;

DROP TABLE IF EXISTS attributes;