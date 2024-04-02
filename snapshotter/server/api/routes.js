import express from "express";
import { GetSnapshot } from "./snapshot.js";

const routes = express.Router();

routes.get("/snapshot", GetSnapshot);

export { routes };
