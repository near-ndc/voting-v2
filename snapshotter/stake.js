// Inspired by https://github.com/zavodil/near-staking-web-tool/blob/main/ndc/stake.js
import pkg from 'pg';
const { Client } = pkg;
import * as nearAPI from "near-api-js";
import { PromisePool } from '@supercharge/promise-pool'
import pRetry from 'p-retry';
import Big from 'big.js'
import fs from 'fs';
import { program } from 'commander';

program
    .description('Load and process staking pools data from NEAR blockchain.')
    .option('--block <type>', 'Block ID to fetch data from', '108194270')
    .option('--dbname <type>', 'Database name', process.env.DB_NAME)
    .option('--user <type>', 'Database user', process.env.DB_USER)
    .option('--password <type>', 'Database password', process.env.DB_PASSWORD)
    .option('--host <type>', 'Database host', process.env.DB_HOST)
    .option('--table <type>', 'Target table name', process.env.TABLE_NAME)
    .option('--column <type>', 'Column name in the table', process.env.COLUMN_NAME)
    .option('--json <type>', 'Path to the JSON file. If it\'s provided, the script will postprocess the data and check the records in the database', undefined);

program.parse(process.argv);
const options = program.opts();

let blockId = 108194270;
const dbParams = {
    database: options.dbname,
    user: options.user,
    password: options.password,
    host: options.host
};
const tableName = options.table;
const columnName = options.column;
const jsonPath = options.json;

const NearConfig = {
    networkId: "mainnet",
    nodeUrl: "https://go.getblock.io/c56f56ef5ae246848ea3e3b66705be4d",
    archivalNodeUrl: "https://rpc.mainnet.internal.near.org",
    walletUrl: "https://wallet.near.org",
};

const keyStore = new nearAPI.keyStores.InMemoryKeyStore();
const _near = {};
_near.nearArchivalConnection = nearAPI.Connection.fromConfig({
    networkId: NearConfig.networkId, provider: {
        type: "JsonRpcProvider", args: { url: NearConfig.archivalNodeUrl },
    }, signer: { type: "InMemorySigner", keyStore },
});

const transformBlockId = (blockId) => blockId === "optimistic" || blockId === "final" ? {
    finality: blockId, blockId: undefined,
} : blockId !== undefined && blockId !== null ? {
    finality: undefined, blockId: parseInt(blockId),
} : {
    finality: "optimistic", blockId: undefined,
};

async function viewCall(provider, blockId, contractId, methodName, args, finality) {
    args = args || {};
    const result = await provider.query({
        request_type: "call_function",
        account_id: contractId,
        method_name: methodName,
        args_base64: Buffer.from(JSON.stringify(args)).toString("base64"),
        block_id: blockId,
        finality,
    });

    return (result.result && result.result.length > 0 && JSON.parse(Buffer.from(result.result).toString()));
}

_near.viewCall = (contractId, methodName, args, blockHeightOrFinality) => {
    const { blockId, finality } = transformBlockId(blockHeightOrFinality);
    return viewCall(_near.nearArchivalConnection.provider, blockId ?? undefined, contractId, methodName, args, finality);
};

const processLockup = async (lockup) => {
    return _near.viewCall(lockup, "get_owner_account_id", {}, blockId)
        .then(owner => {
            console.log(`Lockup ${lockup} owned by ${owner} at block ${blockId}`);
            return owner
        })
        .catch((e) => console.error(e));
};

const processLockups = async (delegators) => {
    let lockupDelegators = Object.keys(delegators).filter(account_id => account_id.endsWith('.lockup.near'));

    const { results: lockupResults, errors: lockupErrors } = await PromisePool
        .withConcurrency(8)
        .for(lockupDelegators)
        .process(async (lockupAccount) => {
            let account_id = await pRetry(() => processLockup(lockupAccount), { retries: 100 });
            return { lockupAccount, account_id };
        });

    if (lockupErrors.length > 0) {
        console.log("Lockup Errors", lockupErrors);
    }

    lockupResults.forEach(result => {
        delegators[result.account_id] = delegators[result.account_id] + delegators[result.lockupAccount];
        delete delegators[result.lockupAccount];
    });

    return delegators;
}

async function checkRecordExists(client, account_id) {
    const query = `SELECT EXISTS(SELECT 1 FROM ${tableName} WHERE ${columnName} = $1)`;
    const res = await client.query(query, [account_id]);
    return res.rows[0].exists;
}

async function loadDelegatorsFromValidators(validators) {
    console.log(`Loading delegators from ${validators.length} validators...`);
    const { results: allValidatorsDetails, errors: poolsError } = await PromisePool
        .withConcurrency(8)
        .for(validators)
        .process(async (accountId) =>
            pRetry(() => _near.viewCall(accountId, "get_number_of_accounts", {}, blockId), { retries: 100 })
                .then(number_of_accounts => ({ account_id: accountId, number_of_accounts }))
        );

    let validatorRequests = [];

    allValidatorsDetails.map(validatorsDetails => {
        console.log(`Validator ${validatorsDetails.account_id} has ${validatorsDetails.number_of_accounts} delegators`)
        for (let i = 0; i < validatorsDetails.number_of_accounts; i += 100) {
            validatorRequests.push({
                account_id: validatorsDetails.account_id,
                from_index: i,
                limit: 100
            });
        }
    });

    const { results: delegators, errors: delegatorsError } = await PromisePool
        .withConcurrency(8)
        .for(validatorRequests)
        .process(async (validatorRequest, index, pool) => {
            const data = await pRetry(() => _near.viewCall(validatorRequest.account_id, "get_accounts", {
                from_index: validatorRequest.from_index,
                limit: validatorRequest.limit
            }, blockId).then((accounts) => {
                console.log(`Loading ${validatorRequest.account_id} delegators: batch #${1 + validatorRequest.from_index / 100}, added ${accounts.length} accounts`)
                return accounts;
            }), { retries: 100 });
            return data;
        });
    if (delegatorsError.length > 0) {
        console.log("Delegators Errors", delegatorsError);
    }

    if (delegators.length === 0) {
        return { results: undefined, errors: poolsError };
    }

    let results = {};
    delegators.map(accounts => {
        accounts.map(account => {
            let stakedBalance = parseFloat(new Big(account.staked_balance).div(new Big(10).pow(24)).toFixed(2));
            if (stakedBalance > 0) {
                let balance = results[account.account_id] ?? 0;
                results[account.account_id] = balance + stakedBalance;
            }
        });
    });

    return { results, errors: poolsError };
}

async function addToDatabase(client, account_id) {
    const query = `INSERT INTO ${tableName} (${columnName}) VALUES ($1)`;
    const res = await client.query(query, [account_id]);
    return res;

}

async function processGaps(accounts, client) {
    // We need to check the gap if it has staking interface:
    // 1. If it has staking interface, we need to get the delegators
    // 2. If it doesn't, we add the account to the database (it may be a gap / or unsupported staking mechanism)
    let { results: newDelegators, errors: newDelegatorsErrors } = await loadDelegatorsFromValidators(accounts.map(account => account.account_id));
    let failedAccoutns = newDelegatorsErrors.map(error => error.item);
    if (newDelegators === undefined) {
        newDelegators = {};
    }
    for (let i in failedAccoutns) {
        let account = accounts[i];
        console.log('Unsupported staking mechanism/missed account: Adding to the database:', account.account_id);
        newDelegators[account.account_id] = account.stake;
        await addToDatabase(client, account.account_id);
    }
    return newDelegators;
}

async function checkAndFixGaps(delegators, client) {
    if (delegators === undefined || Object.keys(delegators).length === 0) {
        return {};
    }

    console.log(`Checking and fixing gaps for ${Object.keys(delegators).length} delegators...`)
    // We resolve lockups first
    let lockuplessDelegators = await processLockups(delegators);

    // Check the database for the records
    const { results: existsResults, errors: existsErrors } = await PromisePool
        .withConcurrency(8)
        .for(Object.keys(lockuplessDelegators))
        .process(async (account_id) => {
            const exists = await checkRecordExists(client, account_id);
            if (!exists) {
                console.log(`Record for '${account_id}' does NOT exist in the database.`);
            }
            return { account_id, exists, stake: lockuplessDelegators[account_id] };
        });
    if (existsErrors.length > 0) {
        console.log("Exists Errors", existsErrors);
    }

    const existedDelegators = existsResults.filter(result => result.exists);
    const potentialDelegators = existsResults.filter(result => !result.exists);

    console.log(`Found ${existedDelegators.length} delegators with records in the database.`);
    console.log(`Found ${potentialDelegators.length} delegators without records in the database.`);

    const processedDelegator = await processGaps(potentialDelegators, client);
    const newDelegators = await checkAndFixGaps(processedDelegator, client);

    let result = {};
    for (let account of Object.keys(newDelegators)) {
        result[account] = newDelegators[account];
    }
    for (let account of existedDelegators) {
        result[account.account_id] = (result[account.account_id] ?? 0) + account.stake;
    }
    return result;
}

let initialDelegators = {};
if (jsonPath === undefined) {
    // Load pools from the chain
    const blockInfo = await _near.nearArchivalConnection.provider.block({ blockId });

    const validators = await _near.nearArchivalConnection.provider.validators(blockInfo.header.epoch_id)
        .then(validators => validators.current_validators.map(validator => validator.account_id));

    console.log(`Loading delegators of ${validators.length} staking pools at block ${blockId}...`);
    const { results, errors } = await loadDelegatorsFromValidators(validators);
    if (errors.length > 0) {
        console.log("Errors", errors);
    }

    initialDelegators = results;
} else {
    // Read the JSON file and extract the names
    const data = JSON.parse(fs.readFileSync(jsonPath, 'utf-8'));
    initialDelegators = data;
}

const client = new Client(dbParams);
await client.connect();

const delegators = await checkAndFixGaps(initialDelegators, client);

console.log("====");
console.log(`${Object.keys(delegators).length} unique delegators found.`);


fs.writeFileSync(`stakes_${blockId}.fixed.json`, JSON.stringify({ ...delegators }));
console.log(`File ${`stakes_${blockId}.fixed.json`} has been updated`);

client.end()
