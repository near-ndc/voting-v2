SET
    work_mem = '2GB';

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
    WHERE
        t.block_height < 108194271
        and t.status = 'SUCCESS_RECEIPT_ID'
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
