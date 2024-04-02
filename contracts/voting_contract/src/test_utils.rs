use std::str::FromStr;

use near_sdk::{test_utils::VMContextBuilder, AccountId, VMContext};

use crate::Contract;

/// 1ms in nano seconds
pub const MSECOND: u64 = 1_000_000;

// In milliseconds
pub const START: u64 = 1;

pub fn acc(idx: u8) -> AccountId {
    AccountId::from_str(&format!("user-{}.near", idx)).unwrap()
}

pub fn relayer() -> AccountId {
    AccountId::from_str("relayer.near").unwrap()
}

pub fn end_time() -> u64 {
    START + 500
}

pub fn setup_ctr() -> (VMContext, Contract) {
    let context = VMContextBuilder::new().build();

    let contract = Contract::new(relayer(), end_time());

    (context, contract)
}
