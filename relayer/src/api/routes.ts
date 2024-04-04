import express from "express";
import { postVote } from "./controllers/submitVote";

const routes = express.Router();

routes.post("/vote", postVote)

export { routes };
