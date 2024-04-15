# voting-contract

The private voting contract for NDC governance

## How to Build Locally?

Install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near build
```

## How to Test Locally?

```bash
cargo test
```

## How to Deploy?

Deployment is automated with GitHub Actions CI/CD pipeline.
To deploy manually, install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near deploy <account-id>
```

## Contract interface

```rust
// Initialization
pub fn new(relayer: AccountId, end_time_in_ms: Timestamp) -> Self

// Relayer methods
pub fn send_encrypted_votes(&mut self, votes: Vec<EncryptedVoteView>)
pub fn sumbit_results(&mut self, results: Vec<(AccountId, u64)>)

// Views
pub fn get_votes(&self, page: u64, limit: u64) -> Vec<EncryptedVoteView>
pub fn get_total_votes(&self) -> u64
pub fn get_candidate_weights(&self, page: u64, limit: u64) -> Vec<(AccountId, u64)>
pub fn get_relayer(&self) -> AccountId
pub fn get_end_time(&self) -> Timestamp
```
