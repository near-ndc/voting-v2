import pkg from 'pg';
const { Client } = pkg;
import fs from 'fs';
import { program } from 'commander';
import assert from 'assert';

program
    .description('Load and process staking pools data from NEAR blockchain.')
    .option('--block <type>', 'Block height of the snapshot', process.env.BLOCK)
    .option('--dbname <type>', 'Database name', process.env.DB_NAME)
    .option('--user <type>', 'Database user', process.env.DB_USER)
    .option('--password <type>', 'Database password', process.env.DB_PASSWORD)
    .option('--host <type>', 'Database host', process.env.DB_HOST)
    .option('--table <type>', 'Target table name', process.env.TABLE_NAME)
    .option('--json <type>', 'Path to the json with the stake data', process.env.JSON_PATH);

program.parse(process.argv);
const options = program.opts();


let blockId = options.block;
const dbParams = {
    database: options.dbname,
    user: options.user,
    password: options.password,
    host: options.host
};
const tableName = options.table;
let jsonPath = options.json;

console.log("Creating snapshot for block", blockId);

const stakeData = JSON.parse(fs.readFileSync(jsonPath, 'utf-8'));

console.log("Loaded stake data for", Object.keys(stakeData).length, "accounts");

const loadActivityData = async (client) => {
    const query = `
        SELECT * from ${tableName}
    `;

    const res = await client.query(query);
    return res.rows;
}

const client = new Client(dbParams);
await client.connect();
const activityData = await loadActivityData(client);
await client.end();

console.log("Loaded activity data for", activityData.length, "accounts");

const activityDataWithStake = activityData.map((activity) => {
    const stake = stakeData[activity.signer_account_id] ?? '0';
    if (stake > 0) {
        console.log(`Account ${activity.account_id} has stake ${stake}`);
    }
    const example_months = activity.example_months.split(',');
    const example_transaction_hashes = activity.example_transaction_hashes.split(',');
    const active_months = parseInt(activity.active_months);
    assert(example_months.length === example_transaction_hashes.length, 'Length of example_months and example_transaction_hashes should be the same');
    assert(example_months.length === active_months, 'Length of example_months should be the same as active_months');

    return {
        account_id: activity.signer_account_id,
        example_months,
        example_transaction_hashes,
        active_months,
        transactions: activity.transactions,
        stake
    }
});

console.log(`Writing snapshot to snapshot-${blockId}.json`);
console.log(activityDataWithStake);

fs.writeFileSync(`snapshot-${blockId}.json`, JSON.stringify({ block_id: blockId, data: activityDataWithStake }));
