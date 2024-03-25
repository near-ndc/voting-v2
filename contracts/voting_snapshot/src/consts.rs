use near_sdk::Gas;

pub const NOT_AUTHORIZED: &str = "Not authorized";
pub const NOT_ELIGIBLE_VOTER: &str = "Not eligible voter";
pub const DIRECT_CALL: &str = "Should be called directly";
pub const CHALLENGE_OVERFLOW: &str = "Overflow on total challenged";

pub const ON_INITIALIZATION_ONLY: &str = "Allowed only during initialization phase";
pub const ON_REGISTRATION_ONLY: &str = "Allowed only during registration phase";
pub const ON_SNAPSHOT_CHALLENGE_ONLY: &str = "Allowed only during snapshot challenge phase";
pub const NOT_ON_SNAPSHOT_CHALLENGE: &str = "Not allowed on snapshot challenge phase";

pub const RESTART_NOT_ALLOWED: &str = "Restart is not allowed";

pub const NO_DEPOSIT: &str = "No deposit found for the user";
pub const EXPECTED_DEPOSIT: &str = "Expected deposit greater than 1 milli NEAR";
pub const EXPECTED_PROMISE_RESULT: &str = "Expected 1 promise result";

pub const STORAGE_LIMIT_EXCEEDED: &str = "Deposit is not enough to cover storage usage";

// Testnet execution shows 3.14 TGas for this function
// As a safety measure, we will use 5 TGas
// https://testnet.nearblocks.io/txns/BDURcv5JibwkYVxy53bQQ2eGqNMRpLnoBwyqC8a4aet8#execution
pub const ON_REFUND_SUCCESS_GAS: Gas = Gas::from_tgas(5);
