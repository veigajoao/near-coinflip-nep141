use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{collections::UnorderedMap, env, AccountId};

use crate::errors::*;
use crate::StorageKey;

// min deposit for storage is 0.25 NEAR
pub const MIN_STORAGE_BALANCE: u128 = 250_000_000_000_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    pub account_id: AccountId,
    // storage checks
    pub storage_deposit: u128,
    pub storage_used: u64,

    // game balances
    pub balances: UnorderedMap<AccountId, u128>,
}

impl Account {
    pub fn new(account_id: &AccountId, initial_deposit: u128) -> Self {
        Self {
            account_id: account_id.clone(),
            storage_deposit: initial_deposit,
            storage_used: 0,
            balances: UnorderedMap::new(StorageKey::AccountBalances {
                account_id: account_id.clone(),
            }),
        }
    }
}

// Implements storage related methods
impl Account {
    /// Returns NEAR necessary to pay for account's storage
    fn storage_usage_cost(&self) -> u128 {
        self.storage_used as u128 * env::storage_byte_cost()
    }

    /// Returns how much deposited NEAR is not being used for storage
    pub fn storage_funds_available(&self) -> u128 {
        let locked = self.storage_usage_cost();
        self.storage_deposit - locked
    }

    /// Asserts there is sufficient amount of $NEAR to cover storage usage.
    fn assert_storage_usage_cost(&self) {
        let storage_usage_cost = self.storage_usage_cost();
        assert!(
            storage_usage_cost <= self.storage_deposit,
            "{}. Needs to deposit {} more yoctoNear",
            ERR_101,
            storage_usage_cost - self.storage_deposit
        );
    }

    pub fn track_storage_usage(&mut self, initial_storage: u64) {
        let final_storage = env::storage_usage();
        if final_storage > initial_storage {
            self.storage_used += final_storage - initial_storage;
            self.assert_storage_usage_cost();
        } else {
            self.storage_used -= initial_storage - final_storage;
        }
    }

    pub fn deposit_storage_funds(&mut self, deposit: u128) {
        self.storage_deposit += deposit;
    }

    pub fn withdraw_storage_funds(&mut self, withdraw: u128) {
        self.storage_deposit -= withdraw;
    }
}
