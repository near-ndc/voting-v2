SET
    work_mem = '4GB';

CREATE TABLE PoolContracts AS
SELECT
    DISTINCT ra.receipt_receiver_account_id
FROM
    receipt_actions ra
WHERE
    ra.action_kind = 'STAKE'
    and ra.block_height < 108194271;
