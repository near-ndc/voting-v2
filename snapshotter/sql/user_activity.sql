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
)
SELECT
    ds.signer_account_id,
    COUNT(
        DISTINCT FORMAT_DATE('%Y%m', t.block_date)
    ) as active_months
FROM
    DistinctStakedSigners ds
    JOIN transactions t ON ds.signer_account_id = t.signer_account_id
GROUP BY
    ds.signer_account_id;
