# secretContract

The smart contract stores the secret for the voting encryption. 
The contract generates secret and makes available public key to create shared secret.
After the defined time pass, the contract will reveal the secret key to reveal the results.

## Prerequisites

package binaryen, gnumake, rust

## How to Build Locally?

```bash
make build-mainnet
```

## How to Test Locally?

```bash
cargo test
```

## How to Deploy?

```bash
# Change secret mnemonic accordingly
export SECRET_MNEMONIC="PUT YOUR MNEMONIC"

npm i
npm node/deploy.js

# Save CODE_HASH, SECRET_ADDRESS
```

## Key fetching

```bash
# replace with your own contract
export CODE_HASH="88bf17e5a2b4feb74b486740ea3429f08367cfd4e8091a3e757010f56d03f9cf"
export SECRET_ADDRESS="secret1ywmutdph8fjxg43hwtz3chgrymlu5mv2zw96f0"

node node/get_keys.js
```

## Contract interface

```rust
// Initialize secret and put reveal date
pub fn instantiate(self, timestamp: Timestamp) -> ()

pub fn get_keys() -> {
    public: Vec<u8>,
    private: Option<Vec<u8>>,
    end_time: Timestamp
}
```
