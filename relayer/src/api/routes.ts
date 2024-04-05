import express from "express";
import { postVote } from "./controllers/submitVote";
import { getPublicKey, postDecryption } from "./controllers/decryption";

const routes = express.Router();

routes.get("/encryption-public-key", getPublicKey);

routes.post("/vote", postVote)
routes.post("/decrypt", postDecryption);

export { routes };
