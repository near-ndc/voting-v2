use near_sdk::{env, Promise, StorageUsage};

pub mod events;

pub fn finalize_storage_check(
    storage_start: StorageUsage,
    additional_storage: StorageUsage,
) -> bool {
    let user_deposit = env::attached_deposit();
    let storage_used = env::storage_usage()
        .saturating_sub(storage_start)
        .saturating_add(additional_storage);
    let diff = env::storage_byte_cost()
        .checked_mul(storage_used as u128)
        .and_then(|cost| user_deposit.checked_sub(cost));

    if let Some(diff) = diff {
        if diff.as_yoctonear() > 0 {
            Promise::new(env::predecessor_account_id()).transfer(diff);
        }
        true
    } else {
        false
    }
}
