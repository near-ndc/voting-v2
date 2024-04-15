import express from "express";
import { postVote } from "./controllers/submitVote";
import { getPublicKey } from "./controllers/decryption";

const routes = express.Router();

routes.get("/encryption-public-key", getPublicKey);
routes.post("/vote", postVote)

export { routes };
