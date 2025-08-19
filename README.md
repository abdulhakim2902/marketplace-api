# NFT Aggregator

## Pre-requisites

- Install [Rust](https://www.rust-lang.org/tools/install)
- Install [CMake](https://cmake.org/download/)

## Data Migration

Refer to the [sqlx documentation](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md) for more
information.

### Install sqlx-cli

```shell
cargo install sqlx-cli
```

### Create a new migration

```shell
sqlx migrate add -r <migration-name>
```

### Run the migrations

```shell
sqlx migrate run
```

### Revert all migrations

```shell
sqlx migrate revert --target-version 0
```

### Saving query metadata for type checking

Run this every time you change the SQL queries in the code.

```shell
cargo sqlx prepare
```

## Environment Variables

Create a `.env` file in the `./backend` and add the following environment variables:

```sh
DATABASE_URL="postgresql://tapp:tapp@localhost:5432/tapp"
SQLX_OFFLINE=true
```

## Get Started

To enable the log add  `RUST_LOG=info`

```bash
cargo run --release
```

### Configuration File (`config.yaml`) Explanation

The `config.yaml` file is used to configure the NFT aggregator. Below is an explanation of each field:

- **tapp_url**: Tapp aptos price indexer
- **admin_config**:
  - **user**: The admin username
  - **password**: The admin password
- **server_config**: 
  - **port**: Port number for the endpoint (e.g. 8080)
- **jwt_config**:
  - **secret**: The jwt secret
  - **expires_in**: The jwt expiration
- **db_config**:
    - **pool_size**: PostgreSQL pool size
    - **url**: PostgreSQL connection string. **Replace with your own.**
- **stream_config**:
  - **indexer_grpc**: The gRPC address (e.g., "https://grpc.mainnet.aptoslabs.com:443")
  - **auth_token**: The authentication token. **Replace with your own.**
    Get your token from https://developers.aptoslabs.com/
- **nft_marketplace_configs**: A list of marketplace configurations, each containing:
  - **name**: Marketplace identifier (e.g., "topaz", "tradeport", "bluemove")
  - **starting_version**: The starting version of the marketplace contract
  - **ending_version**: The ending version of the marketplace contract (optional)
  - **contract_address**: Marketplace contract address
  - **event_model_mapping**: List of event type configurations
  - **events**: Mapping event configurations for database tables and their column
    - **nft_marketplace_activities**: Activity table configurations
      - **collection_addr**: Collection identifier
      - **token_addr**: Token data identifier
      - **token_name**: Name of the token
      - **creator_address**: Creator's address
      - **collection_name**: Name of the collection
      - **price**: Price of the NFT
      - **buyer**: Buyer's address
      - **seller**: Seller's address
      - **token_amount**: Amount of tokens
      - **listing_id**: Listing identifier
      - **offer_id**: Offer identifier
      - **expiration_time**: Offer/listing expiration time

Each column configuration can include:
- **path**: JSON path array for extracting values from event data
- **source**: Data source ("events" by default, or "write_set_changes")
- **resource_type**: Required for `write_set_changes`, specifies the resource type (e.g., "0x4::token::Token")
- **event_type**: Optional, specifies which event type requires this field

### Data Processing

The processor handles two types of data:

1. **Events**: Processed by the EventRemapper, which:
  - Matches events to marketplace configurations
  - Extracts data using configured JSON paths
  - Creates NFT marketplace activities
  - Sets token standard (v1 or v2)
  - Generates token_data_id and collection_id if needed

2. **WriteSetChanges**: Processed by the ResourceMapper, which:
  - Matches token_data_id or collection_id to existing activities based on the `resource_type` field of the write_set_changes
  - Updates activities with additional data from resources
  - Handles V2 token standard specific data

### Admin and User Management API

To access the api explorer

``{basepath}/api/v1/docs``
