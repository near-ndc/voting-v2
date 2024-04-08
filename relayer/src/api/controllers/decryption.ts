import { Request, Response } from "express";
import { getSecretKeys } from "../utils/secret";
import { base_encode } from "near-api-js/lib/utils/serialize";
import { getVoterPublicKey, isNominee, getAllVotes, sendResultsToContract } from "../utils/near";
import { decrypt, verifySignature } from "../../cryptography";
import { VotingPackage } from "../../cryptography/types";

export const getPublicKey = async (_: Request, res: Response) => {
    const secretKeys = await getSecretKeys();

    if (secretKeys.error || !secretKeys.data) {
        console.error("Error while getting secret keys", secretKeys.error);
        return res.status(500).send({ message: "Error while getting secret keys" });
    }

    return res.status(200).send(base_encode(secretKeys.data.public));
}

export const postDecryption = async () => {
    const secretKeys = await getSecretKeys();

    if (secretKeys.error || !secretKeys.data) {
        console.error("Error while getting secret keys", secretKeys.error);
        throw new Error("Error while getting secret keys");
    }

    if (secretKeys.data.private === undefined) {
        const endTime = new Date(secretKeys.data.end_time / 1_000_000).toUTCString()
        throw new Error(`Secret key not available yet. Please come after ${endTime} UTC`);
    }

    const encryptedVotes = await getAllVotes();
    if (encryptedVotes.length === 0) {
        throw new Error(`No votes to decrypt`);
    }

    const decryptedVotes = new Map<string, VotingPackage>();
    for (let i = 0; i < encryptedVotes.length; i++) {
        const vote = encryptedVotes[i];
        let result = await decrypt(vote, secretKeys.data.private);

        if (result.error || !result.data) {
            console.log(`Discard vote ${i}: ${result.error}`);
            continue;
        }

        let isValid = await validateVote(result.data, i);
        if (!isValid) {
            continue;
        }

        if (decryptedVotes.has(result.data.accountId)) {
            console.log(`Found new vote for ${result.data.accountId}. Replacing the old vote`);
        }
        decryptedVotes.set(result.data.accountId, result.data);
    }

    const results = new Map<string, number>();
    decryptedVotes.forEach((vote) => {
        vote.votes.forEach((v) => {
            results.set(v.candidate, (results.get(v.candidate) ?? 0) + v.weight);
        });
    });

    if (await sendResultsToContract(Array.from(results.entries()))) {
        throw new Error("Error while submitting results to the contract");
    }
}

const validateVote = async (vote: VotingPackage, voteNumber: number): Promise<boolean> => {
    const { accountId, votes, signature } = vote;

    const voterInfo = await getVoterPublicKey(accountId);
    if (!voterInfo) {
        console.log(`Discard vote ${voteNumber}: Voter is not registered`);
        return false;
    }

    const data = base_encode(JSON.stringify({ accountId, votes }));
    if (!verifySignature(data, voterInfo.public_key, signature)) {
        console.log(`Discard vote ${voteNumber}: Invalid user signature`);
        return false;
    }


    let totalWeightUsed = 0;
    for (let i = 0; i < votes.length; i++) {
        let vote = votes[i];
        if (vote.weight < 0) {
            console.log(`Discard vote ${voteNumber}: Invalid vote weight`);
            return false;
        }
        totalWeightUsed += vote.weight;

        let status = await isNominee(vote.candidate);
        if (!status) {
            console.log(`Discard vote ${voteNumber}: Invalid candidate`);
            return false;
        }
    }

    if (totalWeightUsed > voterInfo.vote_weight) {
        console.log(`Discard vote ${voteNumber}: Vote weight exceeds the voter's total weight`);
        return false;
    }

    return true;
}
