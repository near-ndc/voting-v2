/*
 It's better to create them after data loading
 */
CREATE INDEX idx_receipt_actions_receipt_id ON receipt_actions (receipt_id);

CREATE INDEX idx_receipt_origin_transaction_originated_from_transaction_hash ON receipt_origin_transaction (originated_from_transaction_hash);

CREATE INDEX idx_transactions_signer_account_id ON transactions (signer_account_id);

CREATE INDEX idx_receipt_actions_kind_receiver ON receipt_actions (action_kind, receipt_predecessor_account_id);

CREATE INDEX idx_transactions_signer_account_id_date ON transactions (signer_account_id, block_timestamp_utc);
