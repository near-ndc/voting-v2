CREATE TABLE ActiveMonthsPerSigner AS WITH DistinctStakedSigners AS (
    SELECT
        DISTINCT t.signer_account_id
    FROM
        PoolContracts pc
        JOIN receipt_actions ra ON ra.receipt_predecessor_account_id = pc.receipt_predecessor_account_id
        JOIN receipt_origin_transaction ro ON ro.receipt_id = ra.receipt_id
        JOIN transactions t ON ro.originated_from_transaction_hash = t.transaction_hash
    WHERE
        ra.action_kind = 'FUNCTION_CALL'
        and t.status = 'SUCCESS'
        and ra.args similar to '%"method_name": "(deposit_and_stake|stake)"%'
        and t.block_height < 108194271
)
SELECT
    ds.signer_account_id,
    COUNT(
        DISTINCT TO_CHAR(t.block_date, 'YYYYMM')
    ) as active_months,
    COUNT(t.transaction_hash) as transactions
FROM
    DistinctStakedSigners ds
    JOIN transactions t ON ds.signer_account_id = t.signer_account_id
GROUP BY
    ds.signer_account_id;
