# Snapshotter

These scripts are used to create a snapshot for the NDC voting v2.

## Prerequisites

This application uses near big query dataset to get the activity data.
Given that we have used snapshots for a particular block in the past, we decided to download the data to the local Postgres to make development cheaper.

Required tables:

* transactions
* receipt_actions
* receipt_origin_transaction

You can download the big query dataset to Google Cloud storage and then download the data to the local server. On 18 March 2024, the cost of migration was about $100 + Tax (99% is outbound download), and the dataset was about 3.1TB (including indexes) of uncompressed data.

To simplify loading, we provide a [./db_loader/load.py](script) that loads zips into the Postgres database.

## Creating snapshot

* Create indexes for loaded data. It may take up to 8 hours (and takes about 500GB)
  
  ```bash
  psql -f sql/indexes.sql >> indexes.nohup.log
  ```

* Create initial distinct stakers signers
  
  ```bash
  psql -f sql/distinct_stakers.sql >> distinct.stakers
  ```

* Fetch active delegators from the contract
  
  ```bash
    npm i
    # It loads all validators. Loads all the delegators for validators' contracts.
    # Postprocesses lockups and other contracts that implement staking pool interface.
    # Also, it loads data skipped users into the `distinctstakedsigners` table (e.g., users that only staked through some non-native pools) 
    node snapshotter/stake.js --dbname SOME_DB --user SOME_USER --password SOME_PASS --host 127.0.0.1 --table distinctstakedsigners --block 108194270 --column signer_account_id  > stake.out
  ```

* Process activity data for all users in the `distinctstakedsigners` table
  
  ```bash
  psql -f sql/active_month_per_signer.sql  > active_months.nohup.log
  ```

* Create a snapshot from the data. Please use the .fixed version of JSON. This JSON is postprocessed to parse other contracts and lockups.

  ```bash
  node snapshotter/prepareSnapshot.js --dbname SOME_DB --user SOME_USER --password SOME_PASS --host 127.0.0.1 --table active_months_per_signer --block 108194270 --json stakes_108194270.fixed.json
  ```

* Congratulations, you have created a snapshot.



