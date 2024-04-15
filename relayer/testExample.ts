import { randomBytes } from "crypto";
import VotingPackageBuilder, { VotingPackageEncryptor } from "./src/utils/vote";
import { keyStores } from "near-api-js";
import { privateKeyVerify } from "secp256k1";
import { base_decode } from "near-api-js/lib/utils/serialize";

// you probably need to implement this function more securely
const generateKeyPair = () => {
    let secret;
    do {
        secret = randomBytes(32)
    } while (!privateKeyVerify(secret))

    return secret;
}

const secretPubKey = async (): Promise<Uint8Array> => {
    const pubkey = await fetch("http://localhost:3000/api/encryption-public-key");
    const string = await pubkey.text();

    return base_decode(string);
}

const encryptAndSendPackage = async (votingPackageEncryptor: VotingPackageEncryptor) => {
    const secretPublicKey = await secretPubKey();
    const encryptionKey = generateKeyPair();

    const encrypted = (await votingPackageEncryptor.encryptPackage(encryptionKey, secretPublicKey)).signPackage();

    const response = await fetch("http://localhost:3000/api/vote", {
        method: "POST",
        body: JSON.stringify(encrypted),
        headers: {
            "Content-Type": "application/json"
        }
    });

    if (response.status !== 200) {
        throw new Error(`Error sending vote: ${await response.text()}`);
    }
}

const main = async () => {
    const homedir = require("os").homedir();
    const CREDENTIALS_DIR = ".near-credentials";
    const credentialsPath = require("path").join(homedir, CREDENTIALS_DIR);
    const myKeyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);
    const yurtur = await myKeyStore.getKey("testnet", "yurtur.testnet");

    const pack = new VotingPackageBuilder("yurtur.testnet")
        .addVote("yurtur.testnet", 1)
        .addVote("absurd-jam.testnet", 5)
        .signPackage(yurtur);

    // Send first vote
    await encryptAndSendPackage(pack);

    // Send invalid vote
    const invalidPack = new VotingPackageBuilder("yurtur.testnet")
        .addVote("yurtur.testnet", 1)
        .addVote("absurd-jam.haha", 5)
        .signPackage(yurtur);

    await encryptAndSendPackage(invalidPack);

    // re-vote for yurtur
    const revote = new VotingPackageBuilder("yurtur.testnet")
        .addVote("yurtur.testnet", 6)
        .signPackage(yurtur);

    await encryptAndSendPackage(revote);

    const absurdJam = await myKeyStore.getKey("testnet", "absurd-jam.testnet");
    const absurdJampack = new VotingPackageBuilder("absurd-jam.testnet").addVote("yurtur.testnet", 1).addVote("absurd-jam.testnet", 1).signPackage(absurdJam);

    await encryptAndSendPackage(absurdJampack);
}

main()
