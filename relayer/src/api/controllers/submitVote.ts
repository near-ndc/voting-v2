import { Request, Response } from "express";
import { EncryptedVotingPackageWithProof } from "../../cryptography/types";
import { getVoterPublicKey, sendVoteToContract } from "../utils/near";
import { verifySignature } from "../../cryptography";
import { publicKeyVerify } from "secp256k1";
import { base_decode } from "near-api-js/lib/utils/serialize";

export const postVote = async (req: Request, res: Response) => {
    // Validate the input using Zod
    const parsedData = await EncryptedVotingPackageWithProof.parseAsync(req.body)
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

    if (parsedData.error || !parsedData.data) {
        return res.status(400).json({ error: "Invalid body" });
    }

    const data = parsedData.data;

    let valid = false;
    try {
        valid = publicKeyVerify(base_decode(data.publicKey));
    } finally {
        if (!valid) {
            return res.status(400).json({ error: 'Invalid public key. Expected 65 bytes' });
        }
    }

    // Check if the user is a registered voter in the snapshot contract
    const voterInfo = await getVoterPublicKey(data.accountId);
    if (!voterInfo) {
        return res.status(400).json({ error: 'User is not a registered voter' });
    }

    // Verify the signature
    const isSignatureValid = verifySignature(data.encryptedData + data.publicKey, voterInfo.public_key, data.signature);
    if (!isSignatureValid) {
        return res.status(400).json({ error: 'Invalid signature' });
    }

    try {
        // Send the vote to the voting contract
        if (!await sendVoteToContract(data)) {
            return res.status(500).json({ error: 'Error submitting vote to contract. Please try again later' });
        }

        res.status(200).json({ message: 'Vote submitted successfully' });
    } catch (error) {
        console.error('Error submitting vote:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
}
