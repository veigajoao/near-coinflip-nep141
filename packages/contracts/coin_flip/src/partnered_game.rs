use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::AccountId;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde( crate = "near_sdk::serde")]
pub struct PartneredGame {
    pub partner_owner: AccountId,
    pub blocked: bool,
    #[serde(with = "crate::string")]
    pub house_funds: u128,
    pub partner_token: AccountId,
    #[serde(with = "crate::string")]
    pub partner_fee: u128, // base 10e-5
    #[serde(with = "crate::string")]
    pub partner_balance: u128,
    #[serde(with = "crate::string")]
    pub bet_payment_adjustment: u128, // base 10e-5 - should pass 10_000 for fair game, less to decrease winners prizes
    #[serde(with = "crate::string")]
    pub house_fee: u128,
    #[serde(with = "crate::string")]
    pub max_bet: u128,
    #[serde(with = "crate::string")]
    pub min_bet: u128,
    pub max_odds: u8,
    pub min_odds: u8,
    #[serde(with = "crate::string")]
    pub nft_fee: u128,   // base 10e-5
    #[serde(with = "crate::string")]
    pub owner_fee: u128, // base 10e-5
}
