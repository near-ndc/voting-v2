import { app } from "./api/app";

import dotenv from "dotenv";

dotenv.config();

export const port = process.env.SERVER_PORT || 3000;
export const votingContract = process.env.VOTING_CONTRACT || '';
export const snapshotContract = process.env.SNAPSHOT_CONTRACT || '';

app.listen(port, () => {
    console.log(`Relayer server running on port :${port}`);
    console.log(`------------------`);
});
