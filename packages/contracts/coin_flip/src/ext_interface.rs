use crate::*;

pub const FT_TRANSFER_GAS: u64 = 50_000_000_000_000;
pub const TRANSFER_CALLBACK_GAS: u64 = 50_000_000_000_000;

#[ext_contract(ext_ft)]
pub trait FunglibleToken {
    fn ft_transfer(receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[ext_contract(ext_self)]
pub trait Callbacks {
    fn owner_transfer_callback(token_contract: AccountId, amount: U128);
    fn nft_transfer_callback(token_contract: AccountId, amount: U128);
    fn project_transfer_callback(amount: U128, project_id: String);
    fn house_funds_transfer_callback(amount: U128, project_id: String);
    fn user_transfer_callback(token_contract: AccountId, amount: U128, user_account_id: AccountId);
}

pub fn transfer_token(token_contract: AccountId, receiver: AccountId, amount: u128) -> Promise {
    ext_ft::ft_transfer(
        receiver,
        U128(amount),
        None,
        &token_contract,
        1,
        FT_TRANSFER_GAS,
    )
}

impl Contract {
    pub fn safe_transfer_owner(&self, token_contract: AccountId, amount: u128) -> Promise {
        transfer_token(token_contract.clone(), self.owner_id.clone(), amount).then(
            ext_self::owner_transfer_callback(
                token_contract,
                U128(amount),
                &env::current_account_id(),
                0,
                TRANSFER_CALLBACK_GAS,
            ),
        )
    }

    pub fn safe_transfer_nft(&self, token_contract: AccountId, amount: u128) -> Promise {
        transfer_token(token_contract.clone(), self.nft_account.clone(), amount).then(
            ext_self::owner_transfer_callback(
                token_contract,
                U128(amount),
                &env::current_account_id(),
                0,
                TRANSFER_CALLBACK_GAS,
            ),
        )
    }

    pub fn safe_transfer_project(
        &self,
        token_contract: AccountId,
        amount: u128,
        project_id: String,
        project_owner_id: AccountId,
    ) -> Promise {
        transfer_token(token_contract.clone(), project_owner_id, amount).then(
            ext_self::project_transfer_callback(
                U128(amount),
                project_id,
                &env::current_account_id(),
                0,
                TRANSFER_CALLBACK_GAS,
            ),
        )
    }

    pub fn safe_transfer_house_funds(
        &self,
        token_contract: AccountId,
        amount: u128,
        project_id: String,
        project_owner_id: AccountId,
    ) -> Promise {
        transfer_token(token_contract.clone(), project_owner_id, amount).then(
            ext_self::house_funds_transfer_callback(
                U128(amount),
                project_id,
                &env::current_account_id(),
                0,
                TRANSFER_CALLBACK_GAS,
            ),
        )
    }

    pub fn safe_transfer_user(
        &self,
        token_contract: AccountId,
        amount: u128,
        user_account_id: AccountId,
    ) -> Promise {
        transfer_token(token_contract.clone(), user_account_id.clone(), amount).then(
            ext_self::user_transfer_callback(
                token_contract,
                U128(amount),
                user_account_id,
                &env::current_account_id(),
                0,
                TRANSFER_CALLBACK_GAS,
            ),
        )
    }
}
