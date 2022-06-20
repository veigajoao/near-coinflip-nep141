use crate::*;

// pub trait Callbacks {
//     fn owner_transfer_callback(token_contract: AccountId, amount: U128);
//     fn nft_transfer_callback(token_contract: AccountId, amount: U128);
//     fn project_transfer_callback(token_contract: AccountId, amount: U128, project_id: String);
//     fn user_transfer_callback(token_contract: AccountId, amount: U128, user_account_id: AccountId);
// }

#[near_bindgen]
impl Contract {
    #[private]
    pub fn owner_transfer_callback(&mut self, token_contract: AccountId, amount: U128) {
        match is_promise_success() {
            true => {}
            false => {
                self.owner_balance.insert(&token_contract, &amount.0);
            }
        }
    }

    #[private]
    pub fn nft_transfer_callback(&mut self, token_contract: AccountId, amount: U128) {
        match is_promise_success() {
            true => {}
            false => {
                self.nft_balance.insert(&token_contract, &amount.0);
            }
        }
    }

    #[private]
    pub fn project_transfer_callback(
        &mut self,
        amount: U128,
        project_id: String,
    ) {
        match is_promise_success() {
            true => {}
            false => {
                let mut game = self.internal_get_game(&project_id).unwrap();
                game.partner_balance += amount.0;
                self.internal_update_game(&project_id, &game);
            }
        }
    }

    #[private]
    pub fn house_funds_transfer_callback(
        &mut self,
        amount: U128,
        project_id: String,
    ) {
        match is_promise_success() {
            true => {}
            false => {
                let mut game = self.internal_get_game(&project_id).unwrap();
                game.house_funds += amount.0;
                self.internal_update_game(&project_id, &game);
            }
        }
    }

    #[private]
    pub fn user_transfer_callback(
        &mut self,
        token_contract: AccountId,
        amount: U128,
        user_account_id: AccountId,
    ) {
        match is_promise_success() {
            true => {
                let mut account = self.internal_get_account(&user_account_id).unwrap();
                match account.balances.get(&token_contract) {
                    Some(v) => {
                        if v == 0 {
                            let initial_storage = env::storage_usage();
                            account.balances.remove(&token_contract);
                            self.internal_update_account(&user_account_id, &account);
                            account.track_storage_usage(initial_storage);
                            self.internal_update_account(&user_account_id, &account);
                        }
                    }
                    None => {}
                }
            }
            false => {
                let initial_storage = env::storage_usage();
                let mut account = self.internal_get_account(&user_account_id).unwrap();
                let current_balance = account.balances.get(&token_contract).unwrap_or(0);
                account
                    .balances
                    .insert(&token_contract, &(current_balance + amount.0));
                self.internal_update_account(&user_account_id, &account);
                account.track_storage_usage(initial_storage);
                self.internal_update_account(&user_account_id, &account);
            }
        }
    }
}
