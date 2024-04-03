"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.corsConfig = void 0;
exports.corsConfig = {
    origin: [
        "http://localhost:3000",
        "https://near.org",
        "https://near.social",
    ],
    methods: "GET,OPTION,HEAD",
    optionsSuccessStatus: 200,
};
