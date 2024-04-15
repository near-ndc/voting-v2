import express from "express";
import cors from "cors";
import { corsConfig } from "./config/cors.config";
import { routes } from "./routes";

export const app = express();

app.use(express.urlencoded({ extended: true }));
app.use(express.json());
app.use(cors(corsConfig));

app.use("/api", routes);
