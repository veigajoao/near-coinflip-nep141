use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde", tag = "type")]
pub enum CallType {
    FundGame { game_id: String },
    DepositBalance,
}

#[near_bindgen]
impl Contract {
    #[allow(unreachable_patterns)]
    pub fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> U128 {
        match serde_json::from_str::<CallType>(&msg).expect(ERR_005) {
            CallType::FundGame { game_id } => {
                self.fund_game_house(env::predecessor_account_id(), amount.0, game_id);
                U128(0)
            }
            CallType::DepositBalance => {
                self.user_deposit_balance(sender_id, env::predecessor_account_id(), amount.0);
                U128(0)
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    /// ft_on_transfer
    /// method must:
    /// 1. Assert msg is in correct format
    /// 2. Delegate to correct internal method
    /// fund_game_house
    /// method must:
    /// 1. Assert transferred token is the game's token
    /// 2. Increase house balance in the game
    #[test]
    fn test_ft_on_transfer_fund_game_house() {
        fn closure_generator(is_correct_token: bool, seed: u128) -> impl FnOnce() {
            move || {
                let user = format!("{}.testnet", seed);
                let base_token = format!("{}-token.testnet", seed);
                let wrong_token = format!("{}-token2.testnet", seed);
                let signer = if is_correct_token {
                    base_token.clone()
                } else {
                    wrong_token.clone()
                };
                let context = get_context(vec![], false, 0, 1000, signer);
                testing_env!(context);

                let game_id = "the_game".to_string();
                let amount = 1000;
                let mut contract = sample_contract(seed);

                let game_settings = PartneredGame {
                    partner_owner: "anyone".to_string(),
                    blocked: false,
                    house_funds: 347,
                    partner_token: base_token.clone(),
                    partner_fee: 0,
                    partner_balance: 0,
                    bet_payment_adjustment: 0,
                    house_fee: 0,
                    max_bet: 0,
                    min_bet: 0,
                    max_odds: 0,
                    min_odds: 0,
                    nft_fee: 0,
                    owner_fee: 0,
                };
                contract.games.insert(&game_id, &game_settings);

                let result = contract.ft_on_transfer(
                    user.clone(),
                    U128(amount),
                    json!({"type": "FundGame", "game_id": game_id}).to_string(),
                );

                assert_eq!(
                    contract.games.get(&game_id).unwrap().house_funds,
                    amount + game_settings.house_funds
                );
                assert_eq!(result, U128(0));
            }
        }

        let test_cases = [
            // 1. Assert transferred token is the game's token
            (false, Some(ERR_301.to_string())),
            // 2. Assert enough balance
            (true, None),
        ];

        let mut counter = 0;
        IntoIterator::into_iter(test_cases).for_each(|v| {
            run_test_case(closure_generator(v.0, counter), v.1);
            counter += 1;
            println!("{}", counter);
        });
    }

    /// fund_game_house
    /// method must:
    /// 1. Assert user is registered
    /// 2. Panic Button is not on
    /// 3. Increase user balance in the token
    #[test]
    fn test_ft_on_transfer_user_deposit_balance() {
        fn closure_generator(is_registered_user: bool, panic_button: bool, seed: u128) -> impl FnOnce() {
            move || {
                let user = format!("{}.testnet", seed);
                let base_token = format!("{}-token.testnet", seed);
                
                let caller = if is_registered_user {user.clone()} else {"fake.testnet".to_string()};

                let context = get_context(vec![], false, 0, 1000, base_token.to_string());
                testing_env!(context);

                let starting_balance = 267;
                let amount = 1000;
                let mut contract = sample_contract(seed);
                contract.panic_button = panic_button;

                contract.internal_deposit_storage_account(&user, 1000 * ONE_NEAR);
                let mut account = contract.internal_get_account(&user).unwrap();
                account.balances.insert(&base_token, &starting_balance);
                contract.internal_update_account(&user, &account);

                let result = contract.ft_on_transfer(
                    caller,
                    U128(amount),
                    json!({"type": "DepositBalance", "game_id": "game_id"}).to_string(),
                );

                let new_account = contract.internal_get_account(&user).unwrap();

                assert_eq!(
                    new_account.balances.get(&base_token).unwrap(),
                    amount + starting_balance
                );
                assert_eq!(result, U128(0));
            }
        }

        let test_cases = [
            // 1. Assert transferred token is the game's token
            (false, false, Some(ERR_001.to_string())),
            // 2. Panic Button
            (true, true, Some(ERR_007.to_string())),
            // 3. Increase user balance in the token
            (true, false, None),
        ];

        let mut counter = 0;
        IntoIterator::into_iter(test_cases).for_each(|v| {
            run_test_case(closure_generator(v.0, v.1, counter), v.2);
            counter += 1;
            println!("{}", counter);
        });
    }
}
