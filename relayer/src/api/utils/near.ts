import { connect, Contract, keyStores, ConnectConfig, Account, Near } from 'near-api-js';
import { AccountId, EncryptedVotingPackage } from '../../cryptography/types';
import { NETWORK_ID, RELAYER_ACCOUNT, SNAPSHOT_CONTRACT, VOTING_CONTRACT } from '../..';
import os from 'os';
import path from 'path';
import { parseNearAmount } from 'near-api-js/lib/utils/format';
import PromisePool from '@supercharge/promise-pool';
import pRetry, { FailedAttemptError } from "p-retry";


// Load credentials;
const homedir = os.homedir();
const CREDENTIALS_DIR = ".near-credentials";
const credentialsPath = path.join(homedir, CREDENTIALS_DIR);
const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

// 300TGAS
const GAS = "300000000000000";
const DEPOSIT = parseNearAmount("0.5");
const RETRIES = 20;

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
    is_nominee: (args: { nominee: AccountId }) => Promise<boolean>;
};

type VotingContract = Contract & {
    send_encrypted_votes: (args: any) => Promise<void>;
    sumbit_results: (args: any) => Promise<void>;

    get_total_votes: () => Promise<number>;
    get_votes: (args: { page: number, limit: number }) => Promise<any>;
};

export const initializeNear = async () => {
    try {
        near = await connect(connectionConfig);

        relayer = await near.account(RELAYER_ACCOUNT!);

        snapshotContract = new Contract(relayer, SNAPSHOT_CONTRACT!, {
            viewMethods: ["get_voter_information", "is_nominee"],
            changeMethods: [],
            useLocalViewExecution: false,
        }) as SnapshotContract;

        votingContract = new Contract(relayer, VOTING_CONTRACT!, {
            viewMethods: ['get_total_votes', 'get_votes'],
            changeMethods: ['send_encrypted_votes', 'sumbit_results'],
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
    } catch (_) {
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

export const sendResultsToContract = async (results: [AccountId, number][]): Promise<boolean> => {
    try {
        await votingContract.sumbit_results({ args: { results }, gas: GAS, amount: DEPOSIT });
        return true;
    } catch (error) {
        console.error('Error submitting results to contract:', error);
        return false;
    }
}

export const getAllVotes = async (): Promise<EncryptedVotingPackage[]> => {
    try {
        const totalVotes = await votingContract.get_total_votes();
        const votes: EncryptedVotingPackage[] = [];

        const PAGE_SIZE = 2;
        const totalPages = Math.ceil(totalVotes / PAGE_SIZE);
        const pageNumbers = Array.from({ length: totalPages }, (_, i) => i);

        const onFailedAttempt = (error: FailedAttemptError) => {
            console.warn(`Attempt ${error.attemptNumber} failed. There are ${error.retriesLeft} retries left.`);
        };

        const { results: voteResults, errors: voteErrors } = await PromisePool
            .withConcurrency(10)
            .for(pageNumbers)
            .useCorrespondingResults()
            .process(async (page) => {
                return pRetry(async () => {
                    const pageVotes = await votingContract.get_votes({
                        page,
                        limit: PAGE_SIZE,
                    });

                    return pageVotes.map((vote: any) => ({
                        encryptedData: vote.vote,
                        publicKey: vote.pubkey,
                    }));
                }, {
                    retries: RETRIES,
                    onFailedAttempt,
                });
            });

        votes.push(...voteResults.flat());

        if (voteErrors.length > 0) {
            console.error('Errors occurred while loading votes:', voteErrors);
            return [];
        }

        return votes;
    } catch (error) {
        console.error('Error loading votes from contract:', error);
        return [];
    }
};

export const isNominee = async (accountId: AccountId): Promise<boolean> => {
    try {
        return await snapshotContract.is_nominee({ nominee: accountId });
    } catch (_) {
        return false;
    }
}
