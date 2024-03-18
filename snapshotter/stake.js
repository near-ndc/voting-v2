// Inspired by https://github.com/zavodil/near-staking-web-tool/blob/main/ndc/stake.js
import pkg from 'pg';
const { Client } = pkg;
import * as nearAPI from "near-api-js";
import { PromisePool } from '@supercharge/promise-pool'
import pRetry from 'p-retry';
import Big from 'big.js'
import fs from 'fs';
import { program } from 'commander';

const EMPTY_HASH = '11111111111111111111111111111111'

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

let blockId = options.block;
const dbParams = {
    database: options.dbname,
    user: options.user,
    password: options.password,
    host: options.host
};
const tableName = options.table;
const columnName = options.column;
let jsonPath = options.json;

const NearConfig = {
    networkId: "mainnet",
    nodeUrl: "https://rpc.mainnet.near.org",
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

_near.viewAccount = (accountId, blockHeightOrFinality) => {
    const { blockId, finality } = transformBlockId(blockHeightOrFinality);
    return _near.nearArchivalConnection.provider.query({ request_type: "view_account", account_id: accountId, block_id: blockId, finality });
};

const processLockup = async (lockup) => {
    return _near.viewCall(lockup, "get_owner_account_id", {}, blockId)
        .then(owner => {
            console.log(`Lockup ${lockup} owned by ${owner} at block ${blockId}`);
            return owner
        });
};

const processLockups = async (delegators) => {
    let lockupDelegators = Object.keys(delegators).filter(account_id => account_id.endsWith('.lockup.near'));

    const { results: lockupResults, errors: lockupErrors } = await PromisePool
        .withConcurrency(24)
        .for(lockupDelegators)
        .process(async (lockupAccount) => {
            return pRetry(() => processLockup(lockupAccount).then((account_id) => {
                return { lockupAccount, account_id };
            }), { retries: 500, onFailedAttempt });
        });

    if (lockupErrors.length > 0) {
        console.log("Lockup Errors", lockupErrors);
    }

    lockupResults.forEach(result => {
        delegators[result.account_id] = (delegators[result.account_id] ?? new Big(0)).add(delegators[result.lockupAccount]);
        delete delegators[result.lockupAccount];
    });

    return delegators;
}

async function checkRecordsExist(client, accountIds) {
    const inClause = accountIds.map((value) => `'${value}'`).join(', ');

    const query = `
        SELECT ${columnName} from ${tableName} where ${columnName} IN (${inClause})
    `;

    const res = await client.query(query);
    const exists = res.rows.map(row => row[columnName]);
    return accountIds.map(accountId => ({ account_id: accountId, exists: exists.includes(accountId) }));

}

let recursePrevent = {};

async function loadDelegatorsFromValidators(validators) {
    console.log(`Loading delegators from ${validators.length} validators...`);
    const { results: allValidatorsDetails, errors: poolsError } = await PromisePool
        .withConcurrency(8)
        .for(validators.filter(accountId => recursePrevent[accountId] !== 1))
        .process(async (accountId) => {
            recursePrevent[accountId] = 1;
            return pRetry(() => _near.viewCall(accountId, "get_number_of_accounts", {}, blockId), { shouldRetry: (err) => !err.message.includes("Contract method is not found"), onFailedAttempt, retries: 100 })
                .then(number_of_accounts => ({ account_id: accountId, number_of_accounts }));
        }
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
        .withConcurrency(24)
        .for(validatorRequests)
        .process(async (validatorRequest, index, pool) => {
            const data = await pRetry(() => _near.viewCall(validatorRequest.account_id, "get_accounts", {
                from_index: validatorRequest.from_index,
                limit: validatorRequest.limit
            }, blockId).then((accounts) => {
                console.log(`Loading ${validatorRequest.account_id} delegators: batch #${1 + validatorRequest.from_index / 100}, added ${accounts.length} accounts`)
                return accounts;
            }), { retries: 100, factor: 1, shouldRetry: (err) => !err.message.includes("Contract method is not found"), onFailedAttempt });
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
            let stakedBalance = new Big(account.staked_balance);
            if (stakedBalance > 0) {
                let balance = results[account.account_id] ?? new Big(0);
                results[account.account_id] = balance.add(stakedBalance);
            }
        });
    });

    return { results, errors: poolsError };
}

async function addToDatabase(client, accounts) {
    if (accounts === undefined || accounts.length === 0) {
        return {};
    }
    const accountString = accounts.map((value) => `('${value}')`).join(', ');
    const query = `INSERT INTO ${tableName} (${columnName}) VALUES ${accountString}`;
    const res = await client.query(query);
    return res;

}

const onFailedAttempt = (error) => {
    if (error.attemptNumber > 5) {
        console.log(`Failed attempt for ${error.attemptNumber}: ${error.message}$`)
    }
};

async function processGaps(accountsOrPools, client) {
    if (accountsOrPools === undefined || accountsOrPools.length === 0) {
        return {};
    }

    // Check if user has a contract
    console.log(`Checking ${accountsOrPools.length} accounts...`)
    let { results: accounts, errors } = await PromisePool
        .withConcurrency(24)
        .for(accountsOrPools)
        .process(async (account) =>
            pRetry(() =>
                _near.viewAccount(account.account_id, blockId)
                    .then((data) => { return { ...account, data } }),
                {
                    retries: 100, factor: 1,
                    // Account deleted :( or invalid params it means it's some internal account or something like that (see ..NSLP..)
                    // https://github.com/Narwallets/meta-pool/blob/04b6ed9f53be93b94b17fb8135163be7b25bf710/metapool/src/types.rs#L14
                    shouldRetry: (err) => !err.message.includes(`doesn't exist`) && !err.message.includes("Invalid params:"),
                    onFailedAttempt,

                }
            )
        );
    if (errors.length > 0) {
        console.log("Errors", errors);
    }

    let users = accounts.filter(account => account.data.code_hash === EMPTY_HASH);
    console.log(`Found ${users.length} accounts without a contract.`);
    let pools = accounts.filter(account => account.data.code_hash !== EMPTY_HASH);
    console.log(`Found ${pools.length} accounts with a contract.`);

    let { results: newDelegators, errors: newDelegatorsErrors } = await loadDelegatorsFromValidators(pools.map(account => account.account_id));
    let failedAccoutns = newDelegatorsErrors.map(error => error.item);
    if (newDelegators === undefined) {
        newDelegators = {};
    }
    let added = [];

    for (let account_id of failedAccoutns) {
        let stake = pools.find(pool => pool.account_id === account_id).stake;
        console.log('Unsupported staking mechanism: Adding to the database:', account_id);
        newDelegators[account_id] = (newDelegators[account_id] ?? new Big(0)).add(stake);
        added.push(account_id);
    }
    for (let account of users) {
        console.log('Missed account: Adding to the database:', account.account_id);
        newDelegators[account.account_id] = (newDelegators[account.account_id] ?? new Big(0)).add(account.stake);
        added.push(account.account_id);
    }

    await addToDatabase(client, added);
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
    const existsResults = await checkRecordsExist(client, Object.keys(lockuplessDelegators))
        .then(records => records.map((elem) => { return { ...elem, stake: lockuplessDelegators[elem.account_id] } }));
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
        result[account.account_id] = (result[account.account_id] ?? new Big(0)).add(account.stake);
    }
    return result;
}

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
    Object.keys(results).forEach(key => results[key] = results[key].toString());

    fs.writeFileSync(`stakes_${blockId}.json`, JSON.stringify({ ...results }));
    jsonPath = `stakes_${blockId}.json`;
}

const initialDelegators = JSON.parse(fs.readFileSync(jsonPath, 'utf-8'));
Object.keys(initialDelegators).forEach(key => initialDelegators[key] = new Big(initialDelegators[key]));


const client = new Client(dbParams);
await client.connect();

const delegators = await checkAndFixGaps(initialDelegators, client);

console.log("====");
console.log(`${Object.keys(delegators).length} unique delegators found.`);


fs.writeFileSync(`stakes_${blockId}.fixed.json`, JSON.stringify({ ...delegators }));
console.log(`File ${`stakes_${blockId}.fixed.json`} has been updated`);

client.end()
