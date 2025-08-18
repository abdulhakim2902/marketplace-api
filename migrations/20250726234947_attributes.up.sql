-- Add up migration script here
CREATE TABLE IF NOT EXISTS attributes (
    id UUID PRIMARY KEY NOT NULL,   
    collection_id UUID NOT NULL,
    nft_id UUID NOT NULL,
    attr_type VARCHAR NOT NULL,
    value VARCHAR NOT NULL,
    rarity NUMERIC(11, 10) DEFAULT 1,
    score NUMERIC (20, 10) DEFAULT 0,
    updated_at timestamp(6) WITH time zone DEFAULT NOW() NOT NULL
);

CREATE FUNCTION update_rarity_and_score ()
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
                attributes.collection_id,
                attributes.attr_type,
                attributes.value,
                COUNT(*)::NUMERIC
            FROM attributes
            WHERE attributes.collection_id = NEW.collection_id
              AND attributes.attr_type = NEW.attr_type
              AND attributes.value = NEW.value
            GROUP BY attributes.collection_id, attributes.attr_type, attributes.value
            UNION
            SELECT
                NEW.collection_id,
                NEW.attr_type,
                NEW.value,
                0
        )
    SELECT
        (cac.count + 1) / ctn.count   AS rarity,
        ctn.count / (cac.count + 1)   AS score
    FROM collection_attribute_counts cac
        JOIN collection_total_nfts ctn ON ctn.collection_id = cac.collection_id
    LIMIT 1
    INTO NEW.rarity, NEW.score;

    UPDATE attributes
    SET score = NEW.score,
        rarity = NEW.rarity,
        updated_at = NOW()
    WHERE collection_id = NEW.collection_id
        AND attr_type = NEW.attr_type
        AND value = NEW.value;

    RETURN new;
END;
$$
    LANGUAGE plpgsql;

CREATE TRIGGER attributes_before_insert_update_rarity_and_score
    BEFORE INSERT ON attributes
    FOR EACH ROW
EXECUTE FUNCTION update_rarity_and_score ();