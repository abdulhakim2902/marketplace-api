-- Add up migration script here
CREATE TABLE IF NOT EXISTS nft_attributes (
  collection_id VARCHAR(66) NOT NULL,
  nft_id VARCHAR(66) NOT NULL,
  type VARCHAR DEFAULT NULL,
  value VARCHAR DEFAULT NULL,
  PRIMARY KEY (collection_id, nft_id, type, value)
);

CREATE TABLE IF NOT EXISTS collection_attributes (
  collection_id VARCHAR(66) NOT NULL,
  type VARCHAR NOT NULL,
  value VARCHAR NOT NULL,
  rarity NUMERIC(11, 10) DEFAULT 1,
  score NUMERIC (20, 10) DEFAULT 0,
  PRIMARY KEY (collection_id, type, value)
);

CREATE FUNCTION calculate_rarity ()
    RETURNS TRIGGER
AS $$
BEGIN
    WITH
        collection_total_nfts AS (
            SELECT nfts.collection_id, COUNT(*)::NUMERIC FROM nfts
            WHERE nfts.collection_id = NEW.collection_id
            GROUP BY nfts.collection_id
        ),
        collection_attribute_counts AS (
            SELECT
                nft_attributes.collection_id,
                nft_attributes.type,
                nft_attributes.value,
                COUNT(*)::NUMERIC
            FROM nft_attributes
            WHERE nft_attributes.collection_id = NEW.collection_id
              AND nft_attributes.type = NEW.type
              AND nft_attributes.value = NEW.value
            GROUP BY nft_attributes.collection_id, nft_attributes.type, nft_attributes.value
        )
    INSERT INTO collection_attributes (collection_id, type, value, rarity, score)
    SELECT
        collection_attribute_counts.collection_id,
        collection_attribute_counts.type,
        collection_attribute_counts.value,
        (collection_attribute_counts.count / collection_total_nfts.count)   AS rarity,
        (collection_total_nfts.count / collection_attribute_counts.count)   AS score
    FROM collection_attribute_counts
        JOIN collection_total_nfts ON collection_total_nfts.collection_id = collection_attribute_counts.collection_id
    ON CONFLICT (collection_id, type, value)
        DO UPDATE SET
              rarity = EXCLUDED.rarity,
              score = EXCLUDED.score;
    RETURN new;
END;
$$
    LANGUAGE plpgsql;

CREATE TRIGGER attributes_after_insert_calculate_rarity
    AFTER INSERT ON nft_attributes
    FOR EACH ROW
EXECUTE FUNCTION calculate_rarity ();