"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.app = void 0;
const express_1 = __importDefault(require("express"));
const cors_1 = __importDefault(require("cors"));
const cors_config_1 = require("./config/cors.config");
const routes_1 = require("./routes");
exports.app = (0, express_1.default)();
exports.app.use(express_1.default.urlencoded({ extended: true }));
exports.app.use(express_1.default.json());
exports.app.use((0, cors_1.default)(cors_config_1.corsConfig));
exports.app.use("/api", routes_1.routes);
