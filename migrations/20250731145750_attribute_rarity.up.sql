-- Add up migration script here
CREATE TABLE IF NOT EXISTS attribute_rarities (
    collection_id VARCHAR(66) NOT NULL,
    type VARCHAR NOT NULL,
    value VARCHAR NOT NULL,
    rarity NUMERIC(11, 10) DEFAULT 1,
    score NUMERIC (20, 10) DEFAULT 0,
    PRIMARY KEY (collection_id, type, value)
);

CREATE FUNCTION recalculate_rarity ()
    RETURNS TRIGGER
    AS $$
BEGIN
    WITH 
        collection_total_nfts AS (
            SELECT nfts.collection_id, COUNT(*)::NUMERIC FROM nfts
            WHERE nfts.collection_id = NEW.collection_id
            GROUP BY nfts.collection_id
        ),
        collection_attributes AS (
            SELECT 
                attributes.collection_id, 
                attributes.attr_type                    AS type,
                attributes.value,
                COUNT(*)::NUMERIC
            FROM attributes
            WHERE attributes.collection_id = NEW.collection_id 
                AND attributes.attr_type = NEW.attr_type
                AND attributes.value = NEW.value
            GROUP BY attributes.collection_id, attributes.attr_type, attributes.value
        )
    INSERT INTO attribute_rarities (collection_id, type, value, rarity, score)
    SELECT 
        collection_attributes.collection_id, 
        collection_attributes.type, 
        collection_attributes.value, 
        (collection_attributes.count / collection_total_nfts.count)   AS rarity,
        (collection_total_nfts.count / collection_attributes.count)   AS score
    FROM collection_attributes
        JOIN collection_total_nfts ON collection_total_nfts.collection_id = collection_attributes.collection_id
    ON CONFLICT (collection_id, type, value)
        DO UPDATE SET
            rarity = EXCLUDED.rarity,
            score = EXCLUDED.score;
    RETURN new;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER attributes_after_insert_recalculate_rarity
    AFTER INSERT ON attributes
    FOR EACH ROW
    EXECUTE FUNCTION recalculate_rarity ();