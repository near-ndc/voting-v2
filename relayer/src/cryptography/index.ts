import { ecdh, privateKeyVerify, publicKeyVerify } from 'secp256k1';
import { SIV, PolyfillCryptoProvider } from 'miscreant';
import { EncryptedVotingPackage, VotingPackage } from './types';

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

export const decrypt = async (vote: EncryptedVotingPackage, privateKey: Uint8Array): Promise<Result<VotingPackage>> => {
    const voteData = Buffer.from(vote.encryptedData, 'base64');
    const publicKey = Buffer.from(vote.publicKey, 'base64');

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
            encryptedData: Buffer.from(encryptedData).toString('base64'),
            publicKey: Buffer.from(publicKey).toString('base64')
        }
    };
}
