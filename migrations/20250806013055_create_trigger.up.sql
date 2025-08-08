-- Add up migration script here
CREATE FUNCTION add_collection_id ()
    RETURNS TRIGGER
AS $$
BEGIN
    IF NEW.collection_id IS NULL THEN
        SELECT nfts.collection_id FROM nfts
        WHERE nfts.id = NEW.nft_id
        INTO NEW.collection_id;
    END IF;

    RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE FUNCTION update_collection_sales ()
    RETURNS TRIGGER
    AS $$
BEGIN
    IF NEW.collection_id IS NULL THEN
        SELECT nfts.collection_id FROM nfts
        WHERE nfts.id = NEW.nft_id
        INTO NEW.collection_id;
    END IF;

    IF NEW.tx_type = 'buy' OR NEW.tx_type = 'accept-bid' OR NEW.tx_type = 'accept-collection-bid' THEN
        WITH
            sales AS (
                SELECT
                    activities.collection_id,
                    SUM(activities.price)           AS volume,
                    SUM(activities.usd_price)       AS volume_usd,
                    COUNT(*)                        AS total
                FROM activities
                WHERE activities.tx_type IN ('buy', 'accept-bid', 'accept-collection-bid')
                    AND activities.collection_id = NEW.collection_id
                GROUP BY activities.collection_id
                UNION
                SELECT
                    NEW.collection_id,
                    NEW.price,
                    NEW.usd_price,
                    1
            )
        INSERT INTO collections (id, slug, volume, volume_usd, sales)
        SELECT
            sales.collection_id,
            sales.collection_id,
            SUM(sales.volume),
            SUM(sales.volume_usd),
            SUM(sales.total)
        FROM sales
        GROUP BY sales.collection_id
        ON CONFLICT (id)
            DO UPDATE SET
                volume = EXCLUDED.volume,
                volume_usd = EXCLUDED.volume_usd,
                sales = EXCLUDED.sales;
    END IF;

    RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE FUNCTION update_collection_listings ()
    RETURNS TRIGGER
AS $$
BEGIN
    WITH
        listings AS (
            SELECT
                listings.collection_id,
                MIN(listings.price)         AS floor,
                COUNT(*)                    AS total
            FROM listings
            WHERE listings.listed AND listings.collection_id = NEW.collection_id
            GROUP BY listings.collection_id
            UNION
            SELECT
                NEW.collection_id,
                NULL,
                0
        )
    INSERT INTO collections (id, slug, floor, listed)
    SELECT
        listings.collection_id,
        listings.collection_id,
        listings.floor,
        listings.total
    FROM listings
    LIMIT 1
    ON CONFLICT (id)
        DO UPDATE SET
            floor = EXCLUDED.floor,
            listed = EXCLUDED.listed;
    RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE FUNCTION update_collection_owners ()
    RETURNS TRIGGER
AS $$
BEGIN
    WITH
        nft_owners AS (
            SELECT nfts.collection_id, COUNT(DISTINCT nfts.owner) AS total
            FROM nfts
            WHERE (nfts.burned IS NULL OR NOT nfts.burned)
              AND nfts.collection_id = NEW.collection_id
            GROUP BY nfts.collection_id
        )
    INSERT INTO collections (id, slug, owners)
    SELECT
        NEW.collection_id,
        NEW.collection_id,
        (
            SELECT COUNT(DISTINCT nfts.owner) FROM nfts
            WHERE (nfts.burned IS NULL OR NOT nfts.burned)
              AND nfts.collection_id = NEW.collection_id
            GROUP BY nfts.collection_id
        )
    ON CONFLICT (id)
        DO UPDATE SET owners = EXCLUDED.owners;
    RETURN new;
END;
$$
    LANGUAGE plpgsql;

CREATE TRIGGER bid_before_insert_add_collection_id
    BEFORE INSERT OR UPDATE ON bids
    FOR EACH ROW
    EXECUTE FUNCTION add_collection_id ();

CREATE TRIGGER activity_before_insert_update_collection_sales
    BEFORE INSERT ON activities
    FOR EACH ROW
    EXECUTE FUNCTION update_collection_sales ();

CREATE TRIGGER listings_after_insert_update_collection_listings
    AFTER INSERT OR UPDATE ON listings
    FOR EACH ROW
EXECUTE FUNCTION update_collection_listings ();

CREATE TRIGGER nfts_after_insert_update_collection_owners
    AFTER INSERT OR UPDATE ON nfts
    FOR EACH ROW
EXECUTE FUNCTION update_collection_owners ();