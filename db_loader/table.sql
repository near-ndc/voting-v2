CREATE TABLE IF NOT EXISTS receipt_actions (
    block_date DATE,
    block_height INTEGER,
    block_timestamp bigint,
    block_timestamp_utc timestamp,
    block_hash VARCHAR(44),
    chunk_hash VARCHAR(44),
    shard_id INTEGER,
    index_in_action_receipt INTEGER,
    receipt_id VARCHAR(44),
    args TEXT,
    receipt_predecessor_account_id VARCHAR(64),
    action_kind VARCHAR(44),
    receipt_receiver_account_id VARCHAR(64),
    is_delegate_action BOOLEAN
);

CREATE TABLE IF NOT EXISTS receipt_origin_transaction (
    block_date DATE,
    block_height INTEGER,
    receipt_kind VARCHAR(12),
    receipt_id VARCHAR(44),
    data_id VARCHAR(44),
    originated_from_transaction_hash VARCHAR(44),
    _record_last_updated_utc timestamp
);

CREATE TABLE IF NOT EXISTS transactions (
    block_date DATE,
    block_height INTEGER,
    block_timestamp bigint,
    block_timestamp_utc timestamp,
    block_hash VARCHAR(44),
    chunk_hash VARCHAR(44),
    shard_id INTEGER,
    transaction_hash VARCHAR(44),
    index_in_chunk INTEGER,
    signer_account_id VARCHAR(64),
    signer_public_key VARCHAR(128),
    nonce bigint,
    receiver_account_id VARCHAR(64),
    signature VARCHAR(128),
    status VARCHAR(20),
    converted_into_receipt_id VARCHAR(44),
    receipt_conversion_gas_burnt bigint,
    receipt_conversion_tokens_burnt FLOAT
);
