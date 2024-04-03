"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.snapshotContract = exports.votingContract = exports.port = void 0;
const app_1 = require("./api/app");
const dotenv_1 = __importDefault(require("dotenv"));
dotenv_1.default.config();
exports.port = process.env.SERVER_PORT || 3000;
exports.votingContract = process.env.VOTING_CONTRACT || '';
exports.snapshotContract = process.env.SNAPSHOT_CONTRACT || '';
app_1.app.listen(exports.port, () => {
    console.log(`Relayer server running on port :${exports.port}`);
    console.log(`------------------`);
});
