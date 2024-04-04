import { ecdh, ecdsaSign, ecdsaVerify, privateKeyVerify, publicKeyVerify } from 'secp256k1';
import { SIV, PolyfillCryptoProvider } from 'miscreant';
import { EncryptedVotingPackage, VotingPackage } from './types';
import { KeyPair, PublicKey } from 'near-api-js/lib/utils';
import { base_decode, base_encode } from 'near-api-js/lib/utils/serialize';

const provider = new PolyfillCryptoProvider();

type Result<T> = {
    error: string | undefined;
    data: T | undefined;
}

const verifyKeys = (privateKey: Uint8Array, publicKey: Uint8Array): boolean => {
    try {
        return publicKeyVerify(publicKey) && privateKeyVerify(privateKey);
    } catch (_) {
        return false;
    }
}

export const verifySignature = (data: string, public_key: string, signature: string): boolean => {
    const message = base_decode(data);
    const signatureBytes = base_decode(signature);
    const [type, key] = public_key.split(':');


    try {
        if (type === "secp256k1") {
            // secp256k1
            const pubkey = base_decode(key);
            return ecdsaVerify(signatureBytes, message, pubkey);
        } else if (type === "ed25519") {
            return PublicKey.from(public_key).verify(message, signatureBytes);
        }
        return false;
    } catch (error) {
        return false;
    }
}

export const createSignature = (data: string, privateStr: string): string | undefined => {
    const message = base_decode(data);
    const [type, private_key] = privateStr.split(':');

    let signature: Uint8Array;
    try {
        if (type === "secp256k1") {
            // secp256k1
            const privKey = base_decode(private_key);
            signature = ecdsaSign(message, privKey).signature;
        } else if (type === "ed25519") {
            // ed25519
            const keyPair = KeyPair.fromString(privateStr);
            signature = keyPair.sign(message).signature;
        }
        else {
            return undefined;
        }
    } catch {
        return undefined;
    }

    return base_encode(signature);
}

export const decrypt = async (vote: EncryptedVotingPackage, privateKey: Uint8Array): Promise<Result<VotingPackage>> => {
    const voteData = base_decode(vote.encryptedData);
    const publicKey = base_decode(vote.publicKey);

    if (!verifyKeys(privateKey, publicKey)) {
        return {
            error: 'Invalid key',
            data: undefined
        };
    }

    const sharedSecret = ecdh(publicKey, privateKey);

    let decryptedData;
    try {
        const siv = await SIV.importKey(sharedSecret, "AES-SIV", provider);
        decryptedData = await siv.open(voteData, []);
    } catch (error) {
        return {
            error: 'Could not decrypt data',
            data: undefined,
        };
    }

    return VotingPackage.parseAsync(JSON.parse(Buffer.from(decryptedData).toString('utf-8')))
        .then((data) => {
            return {
                error: undefined,
                data,
            };
        })
        .catch((_error) => {
            return {
                error: 'Invalid data',
                data: undefined,
            };
        });
}

export const encrypt = async (vote: VotingPackage, privateKey: Uint8Array, publicKey: Uint8Array): Promise<Result<EncryptedVotingPackage>> => {
    if (!verifyKeys(privateKey, publicKey)) {
        return {
            error: 'Invalid key',
            data: undefined
        };
    }

    const sharedSecret = ecdh(publicKey, privateKey);

    const siv = await SIV.importKey(sharedSecret, "AES-SIV", provider);
    const encryptedData = await siv.seal(Buffer.from(JSON.stringify(vote)), []);

    return {
        error: undefined,
        data: {
            encryptedData: base_encode(encryptedData),
            publicKey: base_encode(publicKey)
        }
    };
}
