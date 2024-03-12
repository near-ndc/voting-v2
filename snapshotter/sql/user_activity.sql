CREATE TABLE DistinctStakedSigners AS 
    SELECT
        DISTINCT t.signer_account_id
    FROM
        PoolContracts pc
        JOIN receipt_actions ra ON ra.receipt_receiver_account_id = pc.receipt_predecessor_account_id
        JOIN receipt_origin_transaction ro ON ro.receipt_id = ra.receipt_id
        JOIN transactions t ON ro.originated_from_transaction_hash = t.transaction_hash
    WHERE
        /* the last block that receives 103 confirmations before the 00:00 PST on the 17.12.2023. */
        ra.block_height < 108194271
        and ra.action_kind = 'FUNCTION_CALL'
        and t.status = 'SUCCESS_RECEIPT_ID'
        and ra.args similar to '%"method_name": "(deposit_and_stake|stake)"%'
    GROUP BY
        t.signer_account_id;

CREATE TABLE Active_Months_Per_Signer AS WITH TransactionsWithRowNumber AS (
    SELECT
        t.signer_account_id,
        t.transaction_hash,
        TO_CHAR(t.block_date, 'YYYYMM') as formatted_date,
        ROW_NUMBER() OVER (
            PARTITION BY t.signer_account_id,
            TO_CHAR(t.block_date, 'YYYYMM')
            ORDER BY
                t.block_date
        ) as rn
    FROM
        transactions t
        JOIN DistinctStakedSigners ds ON ds.signer_account_id = t.signer_account_id
)
SELECT
    ds.signer_account_id,
    COUNT(DISTINCT tr.formatted_date) as active_months,
    COUNT(tr.transaction_hash) as transactions,
    /* One transaction hash per each active month */
    STRING_AGG(
        CASE
            WHEN tr.rn = 1 THEN tr.transaction_hash
            ELSE NULL
        END,
        ','
    ) as example_transaction_hashes,
    STRING_AGG(
        CASE
            WHEN tr.rn = 1 THEN tr.formatted_date
            ELSE NULL
        END,
        ','
    ) as example_months
FROM
    DistinctStakedSigners ds
    JOIN TransactionsWithRowNumber tr ON ds.signer_account_id = tr.signer_account_id
GROUP BY
    ds.signer_account_id;