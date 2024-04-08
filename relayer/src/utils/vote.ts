import { base_encode } from "near-api-js/lib/utils/serialize";
import { createSignature, encrypt } from "../cryptography";
import { AccountId, EncryptedVotingPackage, EncryptedVotingPackageWithProof, VotingPackage } from "../cryptography/types";
import { KeyPair } from "near-api-js";

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
    signPackage(keyPair: KeyPair): VotingPackageEncryptor {
        const votes = Array.from(this.votes.entries()).map(([candidate, weight]) => ({ candidate, weight }));

        const signature = createSignature(base_encode(JSON.stringify({ accountId: this.accountId, votes })), keyPair);

        if (!signature) {
            throw new Error("Failed to sign the voting package");
        }

        return new VotingPackageEncryptor(keyPair, {
            accountId: this.accountId,
            votes,
            signature,
        }, this.accountId);
    }
}

export class VotingPackageEncryptor {
    private keyPair: KeyPair;
    private vpackage: VotingPackage;
    private accountId: AccountId;

    constructor(keyPair: KeyPair, vpackage: VotingPackage, accountId: AccountId) {
        this.keyPair = keyPair;
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

        return new EncryptedVotingPackageSigner(this.keyPair, encryptedData.data, this.accountId);
    }

}

export class EncryptedVotingPackageSigner {
    private keyPair: KeyPair;
    private vpackage: EncryptedVotingPackage;
    private accountId: AccountId;

    constructor(keyPair: KeyPair, vpackage: EncryptedVotingPackage, accountId: AccountId) {
        this.keyPair = keyPair;
        this.vpackage = vpackage;
        this.accountId = accountId;
    }

    /// Signs the encrypted voting package
    signPackage(): EncryptedVotingPackageWithProof {
        const signature = createSignature(this.vpackage.encryptedData + this.vpackage.publicKey, this.keyPair);

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
