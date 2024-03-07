CREATE TABLE IF NOT EXISTS receipt_actions (
    block_date DATE NOT NULL,
    block_height INTEGER NOT NULL,
    block_timestamp bigint NOT NULL,
    block_timestamp_utc timestamp NOT NULL,
    block_hash VARCHAR(44) NOT NULL,
    chunk_hash VARCHAR(44) NOT NULL,
    shard_id INTEGER NOT NULL,
    index_in_action_receipt INTEGER NOT NULL,
    receipt_id VARCHAR(44) NOT NULL,
    args TEXT NOT NULL,
    receipt_predecessor_account_id VARCHAR(64) NOT NULL,
    action_kind VARCHAR(44) NOT NULL,
    receipt_receiver_account_id VARCHAR(64) NOT NULL,
    is_delegate_action BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS receipt_origin_transaction (
    block_date DATE NOT NULL,
    block_height INTEGER NOT NULL,
    receipt_kind VARCHAR(12) NOT NULL,
    receipt_id VARCHAR(44) NOT NULL,
    data_id VARCHAR(44),
    originated_from_transaction_hash VARCHAR(44) NOT NULL,
    _record_last_updated_utc timestamp NOT NULL
);

CREATE TABLE IF NOT EXISTS transactions (
    block_date DATE NOT NULL,
    block_height INTEGER NOT NULL,
    block_timestamp bigint NOT NULL,
    block_timestamp_utc timestamp NOT NULL,
    block_hash VARCHAR(44) NOT NULL,
    chunk_hash VARCHAR(44) NOT NULL,
    shard_id INTEGER NOT NULL,
    transaction_hash VARCHAR(44) NOT NULL,
    index_in_chunk INTEGER NOT NULL,
    signer_account_id VARCHAR(64) NOT NULL,
    signer_public_key VARCHAR(64) NOT NULL,
    nonce bigint NOT NULL,
    receiver_account_id VARCHAR(64) NOT NULL,
    signature VARCHAR(128) NOT NULL,
    status VARCHAR(20) NOT NULL,
    converted_into_receipt_id VARCHAR(44) NOT NULL,
    receipt_conversion_gas_burnt bigint NOT NULL,
    receipt_conversion_tokens_burnt FLOAT NOT NULL
);
