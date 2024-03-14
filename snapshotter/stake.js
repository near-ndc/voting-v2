// Inspired by https://github.com/zavodil/near-staking-web-tool/blob/main/ndc/stake.js

import * as nearAPI from "near-api-js";
import { PromisePool } from '@supercharge/promise-pool'
import pRetry from 'p-retry';
import Big from 'big.js'
import fs from 'fs';

let blockId = 108194270; // <-- UPDATE VALUE FOR A GIVEN BLOCK HERE

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

const blockInfo = await _near.nearArchivalConnection.provider.block({
    blockId
});

const validators = await _near.nearArchivalConnection.provider.validators(blockInfo.header.epoch_id)
    .then(validators => validators.current_validators.map(validator => validator.account_id));

console.log(`Loading delegators of ${validators.length} staking pools at block ${blockId}...`);

const loadDelegatorsNumberPromises =
    validators.map(accountId => _near.viewCall(accountId, "get_number_of_accounts", {}, blockId)
        .then(number_of_accounts => ({ account_id: accountId, number_of_accounts }))
        .catch((e) => console.error(e))
    );

const processLockups = async (lockup) => {
    return _near.viewCall(lockup, "get_owner_account_id", {}, blockId)
        .then(owner => {
            console.log(`Lockup ${lockup} owned by ${owner} at block ${blockId}`);
            return owner
        })
        .catch((e) => console.error(e));
};

Promise.all(loadDelegatorsNumberPromises).then(async allValidatorsDetails => {
    let validatorRequests = [];

    allValidatorsDetails.map(validatorsDetails => {
        for (let i = 0; i < validatorsDetails.number_of_accounts; i += 100) {
            validatorRequests.push({
                account_id: validatorsDetails.account_id,
                from_index: i,
                limit: 100
            });
        }
    });

    const { results, errors } = await PromisePool
        .withConcurrency(8)
        .for(validatorRequests)
        .process(async (validatorRequest, index, pool) => {
            const data = await pRetry(() => _near.viewCall(validatorRequest.account_id, "get_accounts", {
                from_index: validatorRequest.from_index,
                limit: validatorRequest.limit
            }, blockId).then((accounts) => {
                console.log(`Loading ${validatorRequest.account_id} delegators: batch #${1 + validatorRequest.from_index / 100}, added ${accounts.length} accounts`)
                return accounts;
            }), { retries: 10 });
            return data;
        });

    if (errors.length > 0) {
        console.log("Errors", errors);
    }

    let totalStake = 0;
    let allDelegators = [];
    results.map(accounts => {
        accounts.map(account => {
            let stakedBalance = parseFloat(new Big(account.staked_balance).div(new Big(10).pow(24)).toFixed(2));
            if (stakedBalance > 0) {
                totalStake += stakedBalance;
                let balance = allDelegators[account.account_id] ?? 0;
                allDelegators[account.account_id] = balance + stakedBalance;
            }
        });
    });

    let lockupDelegators = Object.keys(allDelegators).filter(account_id => account_id.endsWith('.lockup.near'));

    const { results: lockupResults, errors: lockupErrors } = await PromisePool
        .withConcurrency(8)
        .for(lockupDelegators)
        .process(async (lockupAccount) => {
            let account_id = await pRetry(() => processLockups(lockupAccount), { retries: 10 });
            return { lockupAccount, account_id };
        });

    if (lockupErrors.length > 0) {
        console.log("Lockup Errors", lockupErrors);
    }

    // Update allDelegators with lockup names resolved
    lockupResults.forEach(result => {
        allDelegators[result.account_id] = allDelegators[result.account_id] + allDelegators[result.lockupAccount];
        delete allDelegators[result.lockupAccount];
    });

    console.log("====");
    console.log(`${Object.keys(allDelegators).length} unique delegators found. Total Staked: ${totalStake.toFixed(2).replace(/\d(?=(\d{3})+\.)/g, '$&,')}`);

    fs.writeFileSync(`stakes_${blockId}.json`, JSON.stringify({ ...allDelegators }));
    console.log(`File ${`stakes_${blockId}.json`} has been updated`);

});
