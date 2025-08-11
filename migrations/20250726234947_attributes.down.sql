-- Add down migration script here
DROP TRIGGER IF EXISTS attributes_before_insert_update_rarity_and_score ON attributes;

DROP FUNCTION IF EXISTS update_rarity_and_score;

DROP TABLE IF EXISTS attributes;