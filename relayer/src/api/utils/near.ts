import { connect, Contract, KeyPair, keyStores, ConnectConfig, Account, Near } from 'near-api-js';
import { AccountId, EncryptedVotingPackage } from '../../cryptography/types';
import { NETWORK_ID, RELAYER_ACCOUNT, SNAPSHOT_CONTRACT, VOTING_CONTRACT } from '../..';
import os from 'os';
import path from 'path';
import { PublicKey } from 'near-api-js/lib/utils';
import { parseNearAmount } from 'near-api-js/lib/utils/format';

// Load credentials;
const homedir = os.homedir();
const CREDENTIALS_DIR = ".near-credentials";
const credentialsPath = path.join(homedir, CREDENTIALS_DIR);
const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

// 300TGAS
const GAS = "300000000000000";
const DEPOSIT = parseNearAmount("0.5");

let connectionConfig: ConnectConfig;
if (NETWORK_ID === "mainnet") {
    connectionConfig = {
        networkId: "mainnet",
        keyStore,
        nodeUrl: "https://rpc.mainnet.near.org",
        walletUrl: "https://wallet.mainnet.near.org",
        helperUrl: "https://helper.mainnet.near.org",
    };
} else {
    connectionConfig = {
        networkId: "testnet",
        keyStore,
        nodeUrl: "https://rpc.testnet.near.org",
        walletUrl: "https://testnet.mynearwallet.com/",
        helperUrl: "https://helper.testnet.near.org",
    };
}

let near: Near;
let relayer: Account;
let snapshotContract: SnapshotContract;
let votingContract: VotingContract;

type SnapshotContract = Contract & {
    get_voter_information: (args: { voter: AccountId }) => Promise<VoterInfo>;
};

type VotingContract = Contract & {
    send_encrypted_votes: (args: any) => Promise<void>;
};

export const initializeNear = async () => {
    try {
        near = await connect(connectionConfig);

        relayer = await near.account(RELAYER_ACCOUNT!);

        snapshotContract = new Contract(relayer, SNAPSHOT_CONTRACT!, {
            viewMethods: ["get_voter_information"],
            changeMethods: [],
            useLocalViewExecution: false,
        }) as SnapshotContract;

        votingContract = new Contract(relayer, VOTING_CONTRACT!, {
            viewMethods: [],
            changeMethods: ['send_encrypted_votes'],
            useLocalViewExecution: false,
        }) as VotingContract;
    } catch (error) {
        console.error('Error initializing NEAR:', error);
        process.exit(1);
    }
};

export type VoterInfo = {
    vote_weight: number;
    public_key: string;
}

// Function to fetch the user's public key from the snapshot contract
export const getVoterPublicKey = async (accountId: AccountId): Promise<VoterInfo | undefined> => {
    try {
        const voterInfo: VoterInfo = await snapshotContract.get_voter_information({ voter: accountId });
        return voterInfo || undefined;
    } catch (error) {
        console.error('Error fetching voter public key:', error);
        return undefined;
    }
};

// ToDo: bulk submission of votes
export const sendVoteToContract = async (encryptedVotingPackage: EncryptedVotingPackage): Promise<boolean> => {
    try {
        await votingContract.send_encrypted_votes({
            args: {
                votes: [{
                    vote: encryptedVotingPackage.encryptedData,
                    pubkey: encryptedVotingPackage.publicKey,
                }],
            }, gas: GAS, amount: DEPOSIT
        });
        return true;
    } catch (error) {
        console.error('Error submitting vote to contract:', error);
        return false;
    }
};
