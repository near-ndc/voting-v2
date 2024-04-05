import { app } from "./api/app";

import dotenv from "dotenv";
import { initializeNear } from "./api/utils/near";

dotenv.config();

export const PORT = process.env.SERVER_PORT || 3000;
export const VOTING_CONTRACT = process.env.VOTING_CONTRACT;
export const SNAPSHOT_CONTRACT = process.env.SNAPSHOT_CONTRACT;
export const NETWORK_ID = process.env.NETWORK_ID || 'testnet';
export const RELAYER_ACCOUNT = process.env.RELAYER_ACCOUNT;

export const SECRET_CONTRACT = process.env.SECRET_CONTRACT;
export const SECRET_CODE_HASH = process.env.SECRET_CODE_HASH;

app.listen(PORT, async () => {
    let missingEnv = false;

    if (!VOTING_CONTRACT || !SNAPSHOT_CONTRACT) {
        console.error('Please provide VOTING_CONTRACT and SNAPSHOT_CONTRACT in the environment variables');
        missingEnv = true;
    }

    if (!SECRET_CONTRACT || !SECRET_CODE_HASH) {
        console.error('Please provide SECRET_CONTRACT and SECRET_CODE_HASH in the environment variables');
        missingEnv = true;
    }

    if (!RELAYER_ACCOUNT) {
        console.error('Please provide RELAYER_ACCOUNT in the environment variables');
        missingEnv = true;
    }

    if (NETWORK_ID !== 'mainnet' && NETWORK_ID !== 'testnet') {
        console.error('NETWORK_ID should be either mainnet or testnet');
        missingEnv = true;
    }

    if (missingEnv) {
        process.exit(1);
    }

    await initializeNear();

    console.log(`Relayer server running on port :${PORT}`);
    console.log(`Voting Contract: ${VOTING_CONTRACT}`);
    console.log(`Snapshot Contract: ${SNAPSHOT_CONTRACT}`);
    console.log(`Network ID: ${NETWORK_ID}`);
    console.log(`Relayer Account: ${RELAYER_ACCOUNT}`);
    console.log(`Secret Contract: ${SECRET_CONTRACT}`);
    console.log(`Secret Code Hash: ${SECRET_CODE_HASH}`);
    console.log(`------------------`);
});
