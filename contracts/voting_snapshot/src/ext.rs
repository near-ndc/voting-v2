use near_sdk::{ext_contract, AccountId};

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn on_refund_success(&mut self, account_id: AccountId);
}
