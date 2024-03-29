import fs from 'fs';
import { program } from 'commander';
import { keyStores, connect, transactions, Contract } from 'near-api-js';
import { parseNearAmount } from 'near-api-js/lib/utils/format.js';
import os from 'os';
import path from 'path';
import { exit } from 'process';
import { BN } from 'bn.js';

program
    .description('Load the snapshot data on the contract during the initialization phase.')
    .option('--contract <type>', 'Contract address to load the snapshot', process.env.CONTRACT)
    .option('--json <type>', 'Path to the json snapshot', process.env.JSON_PATH)
    .option('--network <type>', 'Testnet or Mainnet', process.env.NETWORK)
    .option('--account <type>', 'Account from keystore to use', process.env.ACCOUNT)
    .option('--start <type>', 'Start loading from butch X', 0);

program.parse(process.argv);
const options = program.opts();

// 300TGAS
const GAS = "300000000000000";
const DEPOSIT = parseNearAmount("2");


let contractId = options.contract;
let jsonPath = options.json;
let network = options.network;
let accountId = options.account;
let index = options.start;

const snapshotToContractRecord = (snapshotRecord) => ([snapshotRecord.account_id, {
    active_months: snapshotRecord.active_months,
    stake: snapshotRecord.stake
}])

function chunkArray(array, chunkSize, mapper) {
    const chunks = [];
    for (let i = 0; i < array.length; i += chunkSize) {
        chunks.push(array.slice(i, i + chunkSize).map(
            mapper
        ));
    }
    return chunks;
}

const snapshot = JSON.parse(fs.readFileSync(jsonPath, 'utf-8')).data;
const transactionsChunks = chunkArray(snapshot, 500, snapshotToContractRecord);

// Load credentials;

const homedir = os.homedir();
const CREDENTIALS_DIR = ".near-credentials";
const credentialsPath = path.join(homedir, CREDENTIALS_DIR);
const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

let connectionConfig;

if (network === "Mainnet") {
    connectionConfig = {
        networkId: "mainnet",
        keyStore,
        nodeUrl: "https://rpc.mainnet.near.org",
        walletUrl: "https://wallet.mainnet.near.org",
        helperUrl: "https://helper.mainnet.near.org",
        explorerUrl: "https://nearblocks.io",
    };
} else {
    connectionConfig = {
        networkId: "testnet",
        keyStore,
        nodeUrl: "https://rpc.testnet.near.org",
        walletUrl: "https://testnet.mynearwallet.com/",
        helperUrl: "https://helper.testnet.near.org",
        explorerUrl: "https://testnet.nearblocks.io",
    };
}
const nearConnection = await connect(connectionConfig);

const account = await nearConnection.account(accountId);
const contract = new Contract(account, contractId, {
    changeMethods: ['bulk_load_voters'],
    viewMethods: ['get_status', 'get_total_eligible_users']
});

let status = await contract.get_status();
if (status.Initialization === undefined) {
    console.error("Wrong contract state");
    exit(0);
}
console.log("Start loading data from the snapshot");
console.log(`The contract is at ${status.Initialization} attempt`)

for (let i = index; i < transactionsChunks.length; i++) {
    let args = transactionsChunks[i];
    try {
        const result = await contract.bulk_load_voters({
            voters: args
        }, GAS, DEPOSIT);
        console.log('Loaded butch', i, 'with', result);
    } catch (e) {
        console.log("Error:", e);
        console.log("Error at index: ", i)
        break;
    }
}

const total_on_contract = await contract.get_total_eligible_users();
const total_in_snapshot = snapshot.length;

console.log(`In contract: ${total_on_contract}`);
console.log(`In snapshot: ${total_in_snapshot}`);
