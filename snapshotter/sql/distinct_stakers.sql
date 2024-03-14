SET
    work_mem = '4GB';

SET
    max_parallel_workers_per_gather = 8;

CREATE TABLE DistinctStakedSigners AS
SELECT
    DISTINCT t.signer_account_id
FROM
    PoolContracts pc
    JOIN receipt_actions ra ON ra.receipt_receiver_account_id = pc.receipt_receiver_account_id
    JOIN receipt_origin_transaction ro ON ro.receipt_id = ra.receipt_id
    JOIN transactions t ON ro.originated_from_transaction_hash = t.transaction_hash
WHERE
    /* the last block that receives 103 confirmations before the 00:00 PST on the 17.12.2023. */
    ra.block_height < 108194271
    and ra.action_kind = 'FUNCTION_CALL'
    and t.status = 'SUCCESS_RECEIPT_ID'
    and ra.args similar to '%"method_name": "(deposit_and_stake|stake|stake_all|unstake_all|unstake|new)"%'
GROUP BY
    t.signer_account_id;
