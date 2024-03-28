# votingSnapshot

The smart contract stores the data snapshot, challenges the snapshot, and registers potential voters and candidates for the next election cycle.

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

To deploy manually, install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near deploy <account-id>
```

## Contract interface

```rust
// Initialization
pub fn new(self, admin: AccountId, vote_config: VoteWeightConfig, process_config: SnapshotConfig) -> Promise

// General transactions:
pub fn register_as_voter(self) -> ()
pub fn register_as_voter_with_pubkey(self, public_key: PublicKey) -> ()
pub fn change_public_key(self, public_key: PublicKey) -> ()
pub fn register_as_nominee(self) -> ()
pub fn challenge_snapshot(self) -> ()
pub fn refund_bond(self) -> ()
pub fn try_move_stage(self) -> bool

// Admin methods
pub fn set_vote_config(self, vote_config: VoteWeightConfig) -> ()
pub fn bulk_load_voters(self, voters: Vec<(AccountId, UserData)>) -> ()
pub fn set_snapshot_config(self, process_config: SnapshotConfig) -> ()
pub fn start_challenge(self) -> ()
pub fn restart_to_initialization(self) -> ()

// Views
pub fn get_vote_config(self) -> VoteWeightConfig
pub fn get_process_config(self) -> SnapshotConfig
pub fn get_end_time(self) -> u64
pub fn get_status(self) -> Status
pub fn get_total_challenge(self) -> NearToken
pub fn get_individual_challenge(self, challenger: &AccountId) -> Option<NearToken>
pub fn get_admin(self) -> AccountId
pub fn get_vote_power(self, voter: &AccountId) -> Option<VoteWeight>
pub fn is_voter(self, voter: &AccountId) -> bool
pub fn is_nominee(self, nominee: &AccountId) -> bool
pub fn is_eligible_voter(self, voter: &AccountId) -> bool
pub fn get_voter_information(self, voter: &AccountId) -> VoterInformation
pub fn get_voters_info(self, voters: Vec<AccountId>) -> Vec<(AccountId, VoterInformation)>

// Callbacks:
pub fn on_refund_success(self, account_id: AccountId) -> ()
```
