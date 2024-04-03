export const corsConfig = {
    origin: [
        "http://localhost:3000",
        "https://near.org",
        "https://near.social",
    ],
    methods: "GET,OPTION,HEAD",
    optionsSuccessStatus: 200,
};
