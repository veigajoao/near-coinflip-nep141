use std::convert::{TryFrom, TryInto};

pub use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, UnorderedMap},
    env, ext_contract,
    json_types::{ValidAccountId, U128, U64},
    near_bindgen,
    serde::{Serialize, Deserialize},
    serde_json::{self, json},
    utils::{assert_one_yocto, is_promise_success},
    AccountId, BorshStorageKey, IntoStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

pub use crate::account::Account;
pub use crate::errors::*;
pub use crate::partnered_game::PartneredGame;

mod account;
mod actions;
mod errors;
mod ext_interface;
mod partnered_game;

pub const FRACTIONAL_BASE: u128 = 100_000;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Accounts,
    PartneredGames,
    AccountBalances { account_id: AccountId },
    OwnerFunds,
    NftFunds,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Contract {
    pub owner_id: AccountId,
    pub nft_account: AccountId,
    pub panic_button: bool,

    pub game_count: u128,

    #[serde(skip)]
    pub accounts: LookupMap<AccountId, Account>,
    #[serde(skip)]
    pub games: LookupMap<String, PartneredGame>,
    #[serde(skip)]
    pub nft_balance: UnorderedMap<AccountId, u128>,
    #[serde(skip)]
    pub owner_balance: UnorderedMap<AccountId, u128>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        nft_account: AccountId,
    ) -> Self {
        assert!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "Invalid owner account"
        );
        assert!(!env::state_exists(), "Already initialized");
        let mut contract = Self {
            owner_id,
            nft_account,
            panic_button: false,

            nft_balance: UnorderedMap::new(StorageKey::NftFunds),
            owner_balance: UnorderedMap::new(StorageKey::OwnerFunds),

            game_count: 0,

            accounts: LookupMap::new(StorageKey::Accounts),
            games: LookupMap::new(StorageKey::PartneredGames),
        };
        let contract_address = env::current_account_id();
        let mut contract_account_entry = Account::new(&contract_address, env::account_balance());
        contract.internal_update_account(&contract_address, &contract_account_entry);
        contract_account_entry.track_storage_usage(0);
        contract.internal_update_account(&contract_address, &contract_account_entry);
        contract
    }
}

// account related methods
impl Contract {
    pub fn internal_get_account(&self, account_id: &AccountId) -> Option<Account> {
        self.accounts.get(account_id)
    }

    pub fn internal_update_account(&mut self, account_id: &AccountId, account: &Account) {
        self.accounts.insert(account_id, account);
    }

    pub fn internal_update_account_storage_check(&mut self, account_id: &AccountId, account: Account, initial_storage: u64) {
        let mut account = account;
        self.internal_update_account(account_id, &account);
        account.track_storage_usage(initial_storage);
        self.internal_update_account(account_id, &account);
    }


    pub fn internal_deposit_storage_account(&mut self, account_id: &AccountId, deposit: u128) {
        let account = match self.internal_get_account(account_id) {
            Some(mut account) => {
                account.deposit_storage_funds(deposit);
                account
            }
            None => Account::new(&account_id.clone(), deposit),
        };
        self.accounts.insert(account_id, &account);
    }

    pub fn internal_storage_withdraw_account(
        &mut self,
        account_id: &AccountId,
        amount: u128,
    ) -> u128 {
        let mut account = self.internal_get_account(&account_id).expect(ERR_001);
        let available = account.storage_funds_available();
        assert!(
            available > 0,
            "{}. No funds available for withdraw",
            ERR_101
        );
        let mut withdraw_amount = amount;
        if amount == 0 {
            withdraw_amount = available;
        }
        assert!(
            withdraw_amount <= available,
            "{}. Only {} available for withdraw",
            ERR_101,
            available
        );
        account.withdraw_storage_funds(withdraw_amount);
        self.internal_update_account(account_id, &account);
        withdraw_amount
    }
}

// partnered_game related methods
impl Contract {
    pub fn internal_get_game(&self, code: &String) -> Option<PartneredGame> {
        self.games.get(code)
    }

    pub fn internal_update_game(&mut self, code: &String, game: &PartneredGame) {
        self.games.insert(code, game);
    }
}

// helper methods
impl Contract {
    fn assert_panic_button(&self) {
        assert!(
            !self.panic_button,
            "{}", ERR_007
        );
    }

    fn only_owner(&self) {
        assert_one_yocto();
        assert!(
            env::predecessor_account_id() == self.owner_id,
            "{}", ERR_006
        );
    }
}

mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use near_sdk::serde::{de, Serializer, Deserialize, Deserializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
        where T: Display,
              S: Serializer
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
        where T: FromStr,
              T::Err: Display,
              D: Deserializer<'de>
    {
        String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    pub use near_sdk::MockedBlockchain;
    pub use near_sdk::{testing_env, VMContext, VMConfig, RuntimeFeesConfig};
    pub use std::panic::{UnwindSafe, catch_unwind};

    pub const CONTRACT_ACCOUNT: &str = "contract.testnet";
    pub const NFT_ACCOUNT: &str = "nft.testnet";
    pub const SIGNER_ACCOUNT: &str = "signer.testnet";
    pub const OWNER_ACCOUNT: &str = "owner.testnet";

    pub const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;

    /// This function can be used witha  higher order closure (that outputs
	/// other closures) to iteratively test diffent cenarios for a call
	pub fn run_test_case<F: FnOnce() -> R + UnwindSafe, R>(
		f: F,
		expected_panic_msg: Option<String>,
	) {
		match expected_panic_msg {
			Some(expected) => {
				match catch_unwind(f) {
					Ok(_) => panic!("call did not panic at all"),
					Err(e) => {
						if let Ok(panic_msg) = e.downcast::<String>() {
							assert!(
								panic_msg.contains(&expected),
								"panic messages did not match, found {}",
								panic_msg
							);
						} else {
							panic!("panic did not produce any msg");
						}
					}
				}
			},
			None => {f();},
		}
    }

    pub fn get_context(input: Vec<u8>, is_view: bool, attached_deposit: u128, account_balance: u128, signer_id: AccountId) -> VMContext {
        VMContext {
            current_account_id: CONTRACT_ACCOUNT.to_string(),
            signer_account_id: signer_id.clone(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: signer_id.clone(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    pub fn sample_contract(seed: u128) -> Contract {
        let hash1 = env::keccak256(&seed.to_be_bytes());
		let hash2 = env::keccak256(&hash1[..]);
		let hash3 = env::keccak256(&hash2[..]);
        let hash4 = env::keccak256(&hash3[..]);
        Contract {
            owner_id: OWNER_ACCOUNT.to_string(),
            nft_account: NFT_ACCOUNT.to_string(),
            panic_button: false,
            nft_balance: UnorderedMap::new(hash1),
            owner_balance: UnorderedMap::new(hash2),
            game_count: 0,

            accounts: LookupMap::new(hash3),
            games: LookupMap::new(hash4),
        }
    }

    #[test]
    fn test_constructor() {
        // set up the mock context into the testing environment
        let base_deposit: u128 = 1_000 * ONE_NEAR;

        fn closure_generator(signer: AccountId, deposit: u128, seed: u128) -> impl FnOnce() {
            move || {
                let context = get_context(vec![], false, deposit, 0, signer);
                testing_env!(context);
                // instantiate a contract variable with the counter at zero
                let initialized_value = Contract::new(
                    OWNER_ACCOUNT.to_string(),
                    NFT_ACCOUNT.to_string(),
                );

                let sample_contract = sample_contract(seed);
                assert_eq!(initialized_value.owner_id, sample_contract.owner_id);
                assert_eq!(initialized_value.nft_account, sample_contract.nft_account);
            }
        }

        let test_cases = [
            (OWNER_ACCOUNT.to_string(), base_deposit, None), // assert normal functioning
            // (OWNER_ACCOUNT.to_string(), 0, Some(ERR_101.to_string())), // assert need to deposit balance
        ];

        let mut counter = 0;
        IntoIterator::into_iter(test_cases).for_each(|v| {
        run_test_case(closure_generator(v.0, v.1, counter), v.2);
        println!("A");
        counter += 1;
        });


    }

}