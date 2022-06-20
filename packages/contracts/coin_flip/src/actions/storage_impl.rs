use crate::*;
use crate::account::{MIN_STORAGE_BALANCE};

use near_sdk::{Promise};
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};

/// Implements users storage management for the pool.
#[near_bindgen]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<ValidAccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let initial_storage = env::storage_usage();
        let amount = env::attached_deposit();
        let account_id = account_id
            .map(|a| a.into())
            .unwrap_or_else(|| env::predecessor_account_id());
        let registration_only = registration_only.unwrap_or(false);
        let min_balance = self.storage_balance_bounds().min.0;
        let already_registered = self.accounts.contains_key(&account_id);
        if amount < min_balance && !already_registered {
            panic!("{}", ERR_102);
        }
        if registration_only {
            // Registration only setups the account but doesn't leave space for tokens.
            if already_registered {
                if amount > 0 {
                    Promise::new(env::predecessor_account_id()).transfer(amount);
                }
            } else {
                self.internal_deposit_storage_account(&account_id, min_balance);
                let refund = amount - min_balance;
                if refund > 0 {
                    Promise::new(env::predecessor_account_id()).transfer(refund);
                }
            }
        } else {
            self.internal_deposit_storage_account(&account_id, amount);
        }
        self.internal_get_account(&account_id).unwrap().track_storage_usage(initial_storage);
        self.storage_balance_of(ValidAccountId::try_from(account_id).unwrap())
            .unwrap()
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let amount = amount.unwrap_or(U128(0)).0;
        let withdraw_amount = self.internal_storage_withdraw_account(&account_id, amount);
        Promise::new(account_id.clone()).transfer(withdraw_amount);
        self.storage_balance_of(ValidAccountId::try_from(account_id).unwrap())
            .unwrap()
    }

    // Change behaviour so that account's allocations are stored in
    // their account and not inside each listing
    #[allow(unused_variables)]
    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        if let Some(account_deposit) = self.internal_get_account(&account_id) {

            // TODO: figure out force option logic.
            assert!(
                account_deposit.balances.is_empty(),
                "{}", ERR_103
            );
            self.accounts.remove(&account_id);
            Promise::new(account_id.clone()).transfer(account_deposit.storage_deposit);
            true
        } else {
            false
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: U128(MIN_STORAGE_BALANCE),
            max: None,
        }
    }

    fn storage_balance_of(&self, account_id: ValidAccountId) -> Option<StorageBalance> {
        match self.internal_get_account(&account_id.to_string()) {
            Some(account) => {
                Some(StorageBalance {
                    total: U128(account.storage_deposit),
                    available: U128(account.storage_funds_available())
                })
            },
            None => None
        }
        
    }
}