import express from "express";
import cors from "cors";
import { corsConfig } from "./config/cors.config.js";
import { routes } from "./routes.js";

export const app = express();

app.use(express.urlencoded({ extended: true }));
app.use(express.json());
app.use(cors(corsConfig));

app.use("/api", routes);
