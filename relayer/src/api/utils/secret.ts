import { SecretNetworkClient, Wallet } from "secretjs";
import ecdsa from "secp256k1";
import { NETWORK_ID, SECRET_CODE_HASH, SECRET_CONTRACT } from "../..";
import { Result } from "../../cryptography";


const wallet = new Wallet();

let secretjs: SecretNetworkClient;

if (NETWORK_ID === "mainnet") {
    secretjs = new SecretNetworkClient({
        chainId: "secret-4",
        url: "https://rpc.ankr.com/http/scrt_cosmos",
        wallet: wallet,
        walletAddress: wallet.address,
    });
}
else {
    secretjs = new SecretNetworkClient({
        chainId: "pulsar-3",
        url: "https://lcd.pulsar-3.secretsaturn.net",
        wallet: wallet,
        walletAddress: wallet.address,
    });
}

type SecretResponse = {
    public: number[],
    private: number[] | undefined,
    end_time: number
}

type Response = {
    public: Uint8Array,
    private: Uint8Array | undefined,
    end_time: number
}

type Request = {
    get_keys: {}
}

export async function getSecretKeys(): Promise<Result<Response>> {
    let query = await secretjs.query.compute.queryContract<Request, SecretResponse>({
        contract_address: SECRET_CONTRACT!,
        query: {
            get_keys: {},
        },
        code_hash: SECRET_CODE_HASH,
    });

    if (query.private !== undefined && query.private.length > 0) {
        const derivedPubKey = ecdsa.publicKeyCreate(Uint8Array.from(query.private), true);
        const publicKey = query.public;

        if (!derivedPubKey.every((value, index) => value === publicKey[index])) {
            return {
                error: 'Secret key is invalid',
                data: undefined
            }
        }
    }

    return {
        error: undefined,
        data: {
            public: Uint8Array.from(query.public),
            private: query.private ? Uint8Array.from(query.private) : undefined,
            end_time: query.end_time
        }
    }
};

