import { SecretNetworkClient, Wallet } from "secretjs";
import dotenv from "dotenv";
import ecdsa from "secp256k1";

dotenv.config({ path: "../../.env" });

const wallet = new Wallet(process.env.SECRET_MNEMONIC);

const secretjs = new SecretNetworkClient({
    chainId: "pulsar-3",
    url: "https://lcd.pulsar-3.secretsaturn.net",
    wallet: wallet,
    walletAddress: wallet.address,
});

// secret contract info
let contractCodeHash = process.env.CODE_HASH;
let contractAddress = process.env.SECRET_ADDRESS;

let get_keys = async () => {
    let query = await secretjs.query.compute.queryContract({
        contract_address: contractAddress,
        query: {
            get_keys: {},
        },
        code_hash: contractCodeHash,
    });

    const publicKeyString = toHexString(query.public);
    const privateKeyString = toHexString(query.private ?? []);
    console.log("Public key:", publicKeyString);

    if (privateKeyString !== "0x") {
        console.log("Private key is available now:", privateKeyString);
        console.log("Veryfying private-public key");

        const derivedPubKeyString = toHexString(ecdsa.publicKeyCreate(Uint8Array.from(query.private), true));
        if (derivedPubKeyString !== publicKeyString) {
            console.log("ERROR: Public key is not created from Private key")
        } else {
            console.log("Keys are correct");
        }
    } else {
        console.log("Private key is not available yet");
        console.log("Expected end time is:", new Date(query.end_time / 1_000_000).toLocaleString());
    }

};

function toHexString(byteArray) {
    return "0x" + Array.from(byteArray, function (byte) {
        return ('0' + (byte & 0xFF).toString(16)).slice(-2);
    }).join('')
}

get_keys();
