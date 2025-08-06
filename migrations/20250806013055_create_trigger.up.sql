-- Add up migration script here
CREATE FUNCTION add_collection_id ()
    RETURNS TRIGGER
    AS $$
DECLARE 
    collection_id VARCHAR(66);
BEGIN
    IF NEW.collection_id IS NULL THEN
        SELECT nfts.collection_id FROM nfts
        WHERE nfts.id = NEW.nft_id
        INTO NEW.collection_id;
    END IF;
    RETURN new;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER bid_before_insert_add_collection_id
    BEFORE INSERT OR UPDATE ON bids
    FOR EACH ROW
    EXECUTE FUNCTION add_collection_id ();

CREATE TRIGGER activity_before_insert_add_collection_id
    BEFORE INSERT OR UPDATE ON activities
    FOR EACH ROW
    EXECUTE FUNCTION add_collection_id ();