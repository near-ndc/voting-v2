import { app } from "./api/app.js";
import fs from 'fs';

const port = process.env.SERVER_PORT || 3000;

const configFile = process.env.SNAPSHOT_FILE || 'snapshot.json';
export let snapshot = {}

function loadSnapshot() {
    const data = JSON.parse(fs.readFileSync(configFile, 'utf-8')).data;
    snapshot = data;
}

app.listen(port, () => {
    loadSnapshot();
    console.log(`Dashboard server running on port :${port}`);
    console.log(`------------------`);
});
