-- Add up migration script here
CREATE TABLE IF NOT EXISTS collection_sales (
    collection_id VARCHAR(66) NOT NULL,
    volumes BIGINT DEFAULT 0,
    sales BIGINT DEFAULT 0,
    PRIMARY KEY (collection_id)
);

CREATE FUNCTION update_collection_sales ()
    RETURNS TRIGGER
    AS $$
BEGIN
    IF NEW.tx_type = 'buy' THEN
      WITH 
        sales AS (
            SELECT
                activities.collection_id, 
                SUM(activities.price)           AS volume,
                COUNT(*)                        AS total
            FROM activities
            WHERE activities.tx_type = 'buy' AND activities.collection_id = NEW.collection_id
            GROUP BY activities.collection_id
        )
      INSERT INTO collection_sales (collection_id, volumes, sales)
      SELECT 
          sales.collection_id,
          sales.volume          AS volumes,
          sales.total           AS sales
      FROM sales
      ON CONFLICT (collection_id)
        DO UPDATE SET
            volumes = EXCLUDED.volumes,
            sales = EXCLUDED.sales;
    END IF;

    RETURN new;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER activities_after_insert_update_collection_sales
    AFTER INSERT ON activities
    FOR EACH ROW
    EXECUTE FUNCTION update_collection_sales ();


CREATE TABLE IF NOT EXISTS collection_listings (
    collection_id VARCHAR(66) NOT NULL,
    floor_price BIGINT DEFAULT 0,
    listed BIGINT DEFAULT 0,
    PRIMARY KEY (collection_id)
);

CREATE FUNCTION update_collection_listings ()
    RETURNS TRIGGER
    AS $$
BEGIN
    WITH 
      listings AS (
          SELECT 
              listings.collection_id,
              MIN(listings.price)             AS floor,
              COUNT(*)                        AS total
          FROM listings
          WHERE listings.listed AND listings.collection_id = NEW.collection_id
          GROUP BY listings.collection_id
      )
    INSERT INTO collection_listings (collection_id, floor_price, listed)
    SELECT 
        listings.collection_id,
        listings.floor                           AS floor_price,
        listings.total                           AS listed
    FROM listings
    ON CONFLICT (collection_id)
      DO UPDATE SET
          floor_price = EXCLUDED.floor_price,
          listed = EXCLUDED.listed;

    RETURN new;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER listings_after_insert_update_collection_listings
    AFTER INSERT OR UPDATE ON listings
    FOR EACH ROW
    EXECUTE FUNCTION update_collection_listings ();

CREATE TABLE IF NOT EXISTS collection_owners (
    collection_id VARCHAR(66) NOT NULL,
    owners BIGINT DEFAULT 0,
    PRIMARY KEY (collection_id)
);

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
    INSERT INTO collection_owners (collection_id, owners)
    SELECT 
        nft_owners.collection_id,
        nft_owners.total                      AS owners
    FROM nft_owners
    ON CONFLICT (collection_id)
      DO UPDATE SET owners = EXCLUDED.owners;
    RETURN new;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER nfts_after_insert_update_collection_owners
    AFTER INSERT OR UPDATE ON nfts
    FOR EACH ROW
    EXECUTE FUNCTION update_collection_owners ();