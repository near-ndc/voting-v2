import { publicKeyCreate, privateKeyVerify } from 'secp256k1';
import { encrypt, decrypt } from '../index';
import { VotingPackage } from '../types';
import { randomBytes } from 'crypto';

describe('Encryption and Decryption', () => {
    let privateKey: Uint8Array;
    let publicKey: Uint8Array;
    let userPrivateKey: Uint8Array;
    let userPublicKey: Uint8Array;

    const votingPackage: VotingPackage = {
        accountId: "ABCD.near",
        votes: [
            { candidate: "One.near", weight: 1 },
            { candidate: "Two.near", weight: 2 },
        ],
        signature: 'ahahahaha',
    };

    const generateKeyPair = () => {
        let secret;
        do {
            secret = randomBytes(32)
        } while (!privateKeyVerify(secret))

        return [secret, publicKeyCreate(secret, true)] as const;
    }

    beforeAll(() => {
        [privateKey, publicKey] = generateKeyPair();
        [userPrivateKey, userPublicKey] = generateKeyPair();
    });

    it('should encrypt and decrypt voting package', async () => {
        const encrypted = await encrypt(votingPackage, privateKey, userPublicKey);
        expect(encrypted.error).toBeUndefined();
        expect(encrypted.data).toBeDefined();

        const decrypted = await decrypt(encrypted.data!, privateKey);
        expect(decrypted.error).toBeUndefined();
        expect(decrypted.data).toEqual(votingPackage);
    });

    it('should fail to decrypt with invalid key', async () => {
        const encrypted = await encrypt(votingPackage, privateKey, userPublicKey);
        expect(encrypted.error).toBeUndefined();
        expect(encrypted.data).toBeDefined();

        const [privateKey1, _] = generateKeyPair();
        const decrypted = await decrypt(encrypted.data!, privateKey1);
        expect(decrypted.error).toBe('Could not decrypt data');
        expect(decrypted.data).toBeUndefined();
    });

    it('should fail to decrypt with invalid data', async () => {
        const encrypted = await encrypt(votingPackage, privateKey, userPublicKey);
        expect(encrypted.error).toBeUndefined();
        expect(encrypted.data).toBeDefined();

        let privkey = Uint8Array.from(userPrivateKey).slice(0, 16);
        const decrypted = await decrypt(encrypted.data!, privkey);
        expect(decrypted.error).toBe('Invalid key');
        expect(decrypted.data).toBeUndefined();
    });

    it('should fail encryption with invalid key', async () => {
        const encrypted = await encrypt(votingPackage, privateKey.slice(5), userPublicKey);
        expect(encrypted.error).toBe('Invalid key');
        expect(encrypted.data).toBeUndefined();
    })
});
