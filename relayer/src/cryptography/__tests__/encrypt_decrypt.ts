import { publicKeyCreate, privateKeyVerify, publicKeyVerify } from 'secp256k1';
import { encrypt, decrypt, createSignature } from '../index';
import { VotingPackage } from '../types';
import { randomBytes } from 'crypto';
import { base_encode } from 'near-api-js/lib/utils/serialize';

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

        return [secret, publicKeyCreate(secret, false)] as const;
    }

    beforeAll(() => {
        [privateKey, publicKey] = generateKeyPair();
        [userPrivateKey, userPublicKey] = generateKeyPair();
    });

    it('should encrypt and decrypt voting package', async () => {
        const encrypted = await encrypt(votingPackage, userPrivateKey, publicKey);
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
        console.log(decrypted);
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

    it('data for testing', async () => {
        const pk = "YOUR PK HERE";
        const votingPackageData = {
            accountId: "yurtur.testnet",
            votes: [
                { candidate: "one.near", weight: 1 },
                { candidate: "two.near", weight: 2 },
            ],
        };
        const sig = await createSignature(base_encode(JSON.stringify(votingPackageData)), pk);

        const votingPackage: VotingPackage = { ...votingPackageData, signature: sig! };

        const publicKey = Uint8Array.from([
            3, 96, 130, 5, 147, 144, 11, 228,
            85, 204, 31, 187, 197, 66, 86, 254,
            95, 147, 16, 227, 252, 210, 205, 247,
            47, 174, 222, 147, 137, 125, 4, 90,
            23
        ]);

        expect(publicKeyVerify(publicKey)).toBe(true);

        const encrypted = await encrypt(votingPackage, privateKey, publicKey);
        console.log(encrypted.data);

        expect(encrypted.error).toBeUndefined();
        expect(encrypted.data).toBeDefined();

        const signature = await createSignature(encrypted.data!.encryptedData + encrypted.data!.publicKey, pk);
        console.log(signature);
    })
});
