-- Add down migration script here
DROP TABLE IF EXISTS attribute_rarities;

DROP TRIGGER attributes_after_insert_recalculate_rarity ON attributes;

DROP FUNCTION recalculate_rarity;