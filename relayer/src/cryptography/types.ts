import { z } from 'zod'

export const AccountId = z.string();

export type AccountId = z.infer<typeof AccountId>;

export const Vote = z.object({
    candidate: AccountId,
    weight: z.number(),
});

export type Vote = z.infer<typeof Vote>;

export const VotingPackage = z.object({
    accountId: AccountId,
    votes: z.array(Vote),
    /// Signature of the data using the account's private key to prove that the data was signed by the account
    signature: z.string(),
});

export type VotingPackage = z.infer<typeof VotingPackage>;

export const EncryptedVotingPackage = z.object({
    encryptedData: z.string(),
    /// Public key that was used to encrypt the data
    /// Shoudn't be the same as the public key of the account
    publicKey: z.string(),
});

export type EncryptedVotingPackage = z.infer<typeof EncryptedVotingPackage>;

export const EncryptedVotingPackageWithProof = z.object({
    ...EncryptedVotingPackage.shape,
    /// Proof that the data was signed by registered account
    /// Please note that this is required only for validation purposes but not forwarded to the chain
    signature: z.string(),
    accountId: AccountId,
});

export type EncryptedVotingPackageWithProof = z.infer<typeof EncryptedVotingPackageWithProof>;
