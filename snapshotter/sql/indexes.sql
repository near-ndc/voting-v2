/*
 It's better to create them after data loading
 */
CREATE INDEX idx_receipt_actions_action_kind ON receipt_actions (action_kind);

CREATE INDEX idx_receipt_actions_receipt_id ON receipt_actions (receipt_id);

CREATE INDEX idx_receipt_origin_transaction_originated_from_transaction_hash ON receipt_origin_transaction (originated_from_transaction_hash);

CREATE INDEX idx_transactions_signer_account_id ON transactions (signer_account_id);

CREATE INDEX idx_receipt_actions_kind_receiver ON receipt_actions (action_kind, receipt_predecessor_account_id);

CREATE INDEX idx_transactions_block_date ON transactions (block_date);

CREATE INDEX idx_transactions_status ON transactions (status);

CREATE INDEX idx_transactions_block_height ON transactions (block_height);

CREATE INDEX idx_receipt_origin_transaction_receipt_id ON receipt_origin_transaction (receipt_id);
