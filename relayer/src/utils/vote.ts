import { base_encode } from "near-api-js/lib/utils/serialize";
import { createSignature, encrypt } from "../cryptography";
import { AccountId, EncryptedVotingPackage, EncryptedVotingPackageWithProof, VotingPackage } from "../cryptography/types";

export default class VotingPackageBuilder {
    private accountId: AccountId;
    private votes: Map<AccountId, number>;

    constructor(accountId: AccountId) {
        this.accountId = accountId;
        this.votes = new Map();
    }

    addVote(candidate: AccountId, weight: number): VotingPackageBuilder {
        this.votes.set(candidate, weight);
        return this;
    }

    removeVote(candidate: AccountId): VotingPackageBuilder {
        this.votes.delete(candidate);
        return this;
    }

    /// Builds a voting package with the given encryption key and private key.
    /// Private key should follow next standard: 'ed25519:bs58private_key' or 'secp256k1:bs58private_key'
    signPackage(privateKey: string): VotingPackageEncryptor {
        const votes = Array.from(this.votes.entries()).map(([candidate, weight]) => ({ candidate, weight }));

        const signature = createSignature(base_encode(JSON.stringify({ accountId: this.accountId, votes })), privateKey);

        if (!signature) {
            throw new Error("Failed to sign the voting package");
        }

        return new VotingPackageEncryptor(privateKey, {
            accountId: this.accountId,
            votes,
            signature,
        }, this.accountId);
    }
}

class VotingPackageEncryptor {
    private privateKey: string;
    private vpackage: VotingPackage;
    private accountId: AccountId;

    constructor(privateKey: string, vpackage: VotingPackage, accountId: AccountId) {
        this.privateKey = privateKey;
        this.vpackage = vpackage;
        this.accountId = accountId;
    }

    /// Encrypts the voting package with the user provided private key.
    /// The encryption key should be secp256k1 private key.
    async encryptPackage(encryptionKey: Uint8Array, secretPubKey: Uint8Array): Promise<EncryptedVotingPackageSigner> {
        const encryptedData = await encrypt(this.vpackage, encryptionKey, secretPubKey);
        if (encryptedData.error || !encryptedData.data) {
            throw new Error(`Failed to encrypt the voting package: ${encryptedData.error}`);
        }

        return new EncryptedVotingPackageSigner(this.privateKey, encryptedData.data, this.accountId);
    }

}

class EncryptedVotingPackageSigner {
    private privateKey: string;
    private vpackage: EncryptedVotingPackage;
    private accountId: AccountId;

    constructor(privateKey: string, vpackage: EncryptedVotingPackage, accountId: AccountId) {
        this.privateKey = privateKey;
        this.vpackage = vpackage;
        this.accountId = accountId;
    }

    /// Signs the encrypted voting package
    signPackage(): EncryptedVotingPackageWithProof {
        const signature = createSignature(this.vpackage.encryptedData + this.vpackage.publicKey, this.privateKey);

        if (!signature) {
            throw new Error("Failed to sign the encrypted voting package");
        }

        return {
            ...this.vpackage,
            signature,
            accountId: this.accountId,
        };
    }
}
