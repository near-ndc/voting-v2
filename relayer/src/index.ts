import { app } from "./api/app";

import dotenv from "dotenv";
import { initializeNear } from "./api/utils/near";

dotenv.config();

export const PORT = process.env.SERVER_PORT || 3000;
export const VOTING_CONTRACT = process.env.VOTING_CONTRACT;
export const SNAPSHOT_CONTRACT = process.env.SNAPSHOT_CONTRACT;
export const NETWORK_ID = process.env.NETWORK_ID || 'testnet';
export const RELAYER_ACCOUNT = process.env.RELAYER_ACCOUNT;

app.listen(PORT, async () => {
    if (!VOTING_CONTRACT || !SNAPSHOT_CONTRACT) {
        console.error('Please provide VOTING_CONTRACT and SNAPSHOT_CONTRACT in the environment variables');
        process.exit(1);
    }

    if (!RELAYER_ACCOUNT) {
        console.error('Please provide RELAYER_ACCOUNT in the environment variables');
        process.exit(1);
    }

    if (NETWORK_ID !== 'mainnet' && NETWORK_ID !== 'testnet') {
        console.error('NETWORK_ID should be either mainnet or testnet');
        process.exit(1);
    }

    await initializeNear();

    console.log(`Relayer server running on port :${PORT}`);
    console.log(`Voting Contract: ${VOTING_CONTRACT}`);
    console.log(`Snapshot Contract: ${SNAPSHOT_CONTRACT}`);
    console.log(`Network ID: ${NETWORK_ID}`);
    console.log(`Relayer Account: ${RELAYER_ACCOUNT}`);
    console.log(`------------------`);
});
