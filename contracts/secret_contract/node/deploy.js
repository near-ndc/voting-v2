import { SecretNetworkClient, Wallet } from "secretjs";
import dotenv from "dotenv";
import fs from "fs"

dotenv.config({ path: "../../.env" });

const wallet = new Wallet(process.env.SECRET_MNEMONIC);

const contract_wasm = fs.readFileSync("./contract.wasm.gz");

const secretjs = new SecretNetworkClient({
    chainId: "pulsar-3",
    url: "https://lcd.pulsar-3.secretsaturn.net",
    wallet: wallet,
    walletAddress: wallet.address,
});

// 10 minutes in millis
const DELAY = 10 * 60 * 1000;

// Declare global variables
let codeId;
let contractCodeHash;

let upload_contract = async () => {
    console.log("Starting deployment…");

    let tx = await secretjs.tx.compute.storeCode(
        {
            sender: wallet.address,
            wasm_byte_code: contract_wasm,
            source: "",
            builder: "",
        },
        {
            gasLimit: 4_000_000,
        }
    );

    // console.log(tx);
    codeId = Number(
        tx.arrayLog.find((log) => log.type === "message" && log.key === "code_id")
            .value
    );
    console.log("codeId: ", codeId);

    contractCodeHash = (
        await secretjs.query.compute.codeHashByCodeId({ code_id: codeId })
    ).code_hash;
    console.log(`Contract hash: ${contractCodeHash}`);
};

let instantiate_contract = async () => {
    if (!codeId || !contractCodeHash) {
        throw new Error("codeId or contractCodeHash is not set.");
    }
    console.log("Instantiating contract…");
    let timestamp = (await secretjs.query.tendermint.getLatestBlock({}))?.block?.header?.time;
    if (timestamp === undefined) {
        throw new Error("Couldn't fetch latest timestamp");
    }

    let tx = await secretjs.tx.compute.instantiateContract(
        {
            code_id: codeId,
            sender: wallet.address,
            code_hash: contractCodeHash,
            init_msg: {
                end_time: (Date.parse(timestamp) + DELAY).toString() + "000000"
            },
            label: "ENCRYPT " + Math.ceil(Math.random() * 10000),
        },
        {
            gasLimit: 400_000,
        }
    );

    // console.log(tx);

    //Find the contract_address in the logs
    const contractAddress = tx.arrayLog.find(
        (log) => log.type === "message" && log.key === "contract_address"
    ).value;

    console.log("contract address: ", contractAddress);
};

// Chain the execution using promises
upload_contract()
    .then(() => {
        instantiate_contract();
    })
    .catch((error) => {
        console.error("Error:", error);
    });
