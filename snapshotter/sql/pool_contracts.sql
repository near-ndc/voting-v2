CREATE TABLE PoolContracts AS
SELECT
    DISTINCT ra.receipt_predecessor_account_id
FROM
    receipt_actions ra
WHERE
    ra.action_kind IN ('STAKE');
