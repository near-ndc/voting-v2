# Voting Relayer Server

This is a relayer server for a voting system that interacts with NEAR / Secret Network blockchain contracts. The server provides functionality to submit encrypted votes to the voting contract and perform decryption job after voting ends.

## Prerequisites

Before running the server, make sure you have the following:

- Node.js installed
- NEAR account credentials
- Access to the voting contract, snapshot contract, and secret contract on the NEAR blockchain

## Installation

1. Install the dependencies:

   ```bash
   npm i
   ```

2. Create a `.env` file in the root directory of the project and provide the following environment variables:

    The server configuration is done through environment variables. Make sure to set the following variables in the `.env` file:
    - `SERVER_PORT`: The port on which the server will run (default: 3000).
    - `VOTING_CONTRACT`: The account name of the voting contract.
    - `SNAPSHOT_CONTRACT`: The account name of the snapshot contract.
    - `NETWORK_ID`: The network ID (`mainnet` or `testnet`).
    - `RELAYER_ACCOUNT`: The account name of the relayer. This account should be in your credentials to sign transactions
    - `SECRET_CONTRACT`: The secret contract address
    - `SECRET_CODE_HASH`: The code hash of the secret contract.

## Usage

The server can be run in two modes: `server` and `decrypt`.

### Server Mode

To start the relay server during the voting phase, run the following command:

```bash
npm run build
npm run start:production
```

The server will start running on the specified port (default: 3000) and connect to the NEAR blockchain using the provided environment variables.

### Decrypt Mode

To run the decryption job, use the following command:

```bash
npm run start:decrypt
```
