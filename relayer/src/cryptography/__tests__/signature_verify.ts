import { KeyPair } from 'near-api-js';
import { createSignature, verifySignature } from '../';
import { randomBytes } from 'crypto';
import { privateKeyVerify, publicKeyCreate } from 'secp256k1';
import { base_encode } from 'near-api-js/lib/utils/serialize';

describe('Signature Creation and Verification', () => {
    let secp256k1KeyPair: readonly [string, string]
    let ed25519KeyPair: readonly [string, string];
    let data: string;

    const generateSecp = () => {
        let secret;
        do {
            secret = randomBytes(32)
        } while (!privateKeyVerify(secret))

        return ['secp256k1:' + base_encode(secret), 'secp256k1:' + base_encode(publicKeyCreate(secret, true))] as const;
    }

    const generateEd = () => {
        const keyPair = KeyPair.fromRandom('ed25519');

        return [keyPair.toString(), keyPair.getPublicKey().toString()] as const;
    }

    beforeAll(async () => {
        secp256k1KeyPair = generateSecp();
        ed25519KeyPair = generateEd();

        const randomData = randomBytes(32);
        data = base_encode(randomData);
    });

    it('should create and verify signature for ed25519 key', () => {
        const [secret, publicKey] = ed25519KeyPair;

        const keyPair = KeyPair.fromString(secret);

        const signature = createSignature(data, keyPair);
        expect(signature).toBeDefined();

        const isValid = verifySignature(data, publicKey, signature!);
        expect(isValid).toBe(true);
    });
});
