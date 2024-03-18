import pkg from 'pg';
const { Client } = pkg;
import fs from 'fs';
import { program } from 'commander';

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

const stakeData = JSON.parse(fs.readFileSync(jsonPath, 'utf-8'));

const loadActivityData = async (client) => {
    const query = `
        SELECT * from ${tableName})
    `;

    const res = await client.query(query);
    return res.rows;
}

const client = new Client(dbParams);
const activityData = await loadActivityData(client);
client.end();

const activityDataWithStake = activityData.map((activity) => {
    const stake = stakeData[activity.account_id];
    return {
        ...activity,
        stake: stake ? stake : '0'
    }
});

fs.writeFileSync(`snapshot-${blockId}.json`, JSON.stringify({ block_id: blockId, data: activityDataWithStake }));
