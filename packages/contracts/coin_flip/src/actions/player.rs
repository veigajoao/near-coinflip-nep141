use crate::*;

#[near_bindgen]
impl Contract {
    pub fn retrieve_credits(&mut self, token_contract: AccountId, amount: U128) -> Promise {
        self.assert_panic_button();
        let account_id = env::predecessor_account_id();
        let initial_storage = env::storage_usage();
        let mut account = self.internal_get_account(&account_id).expect(ERR_001);
        let current_balance = account.balances.remove(&token_contract).unwrap_or(0);
        assert!(current_balance >= amount.0, "{}", ERR_401);
        let new_balance = current_balance - amount.0;
        if new_balance > 0 {
            account.balances.insert(&token_contract, &new_balance);
        }
        self.internal_update_account_storage_check(&account_id, account, initial_storage);
        self.safe_transfer_user(token_contract, amount.0, account_id)
    }

    //plays the game, user can choose the game collection to play within, size of the bet,
    //the odds that they eant to take (the smallet the odds, the greater prize).
    //_bet_type is a dummy param for indexers to display the bet choice the user made, but are
    //irrelevant for game logic
    pub fn play(
        &mut self,
        game_code: AccountId,
        bet_size: U128,
        odds: u8,
        _bet_type: String,
    ) -> bool {
        self.assert_panic_button();

        // check that user has credits
        let account_id = env::predecessor_account_id();

        let mut account = self.internal_get_account(&account_id).expect(ERR_001);
        let mut game = self.internal_get_game(&game_code).expect(ERR_002);
        let mut credits = account.balances.get(&game.partner_token).unwrap_or(0);
        assert!(credits >= bet_size.0, "{}", ERR_402);
        assert!(
            bet_size.0 >= game.min_bet,
            "{}. Minimum is {}",
            ERR_403,
            game.min_bet
        );
        assert!(
            bet_size.0 <= game.max_bet,
            "{}. Maximum is {}",
            ERR_404,
            game.max_bet
        );
        assert!(
            odds >= game.min_odds,
            "{}. Minimum is {}",
            ERR_405,
            game.min_bet
        );
        assert!(
            odds <= game.max_odds,
            "{}. Maximum is {}",
            ERR_406,
            game.max_bet
        );

        // charge dev and nft fees
        let mut net_bet = bet_size.0;
        let nft_cut = (net_bet * game.nft_fee) / FRACTIONAL_BASE;
        let owner_cut = (net_bet * game.owner_fee) / FRACTIONAL_BASE;
        let house_cut = (net_bet * game.house_fee) / FRACTIONAL_BASE;
        let partner_cut = (net_bet * game.partner_fee) / FRACTIONAL_BASE;
        net_bet = net_bet - nft_cut - owner_cut - house_cut - partner_cut;
        let nft_balance = self.nft_balance.get(&game.partner_token).unwrap_or(0);
        self.nft_balance
            .insert(&game.partner_token, &(nft_balance + nft_cut));

        let owner_balance = self.owner_balance.get(&game.partner_token).unwrap_or(0);
        self.owner_balance
            .insert(&game.partner_token, &(owner_balance + owner_cut));
        game.house_funds += house_cut;
        game.partner_balance += partner_cut;

        // send off credits
        credits = credits - bet_size.0;
        let rand = *env::random_seed().get(0).unwrap();
        let random_hash = u128::from_be_bytes(
            env::keccak256(&[rand, (self.game_count % 256) as u8])[0..16]
                .try_into()
                .unwrap(),
        );
        let rand_shuffled = (random_hash % 256) as u8;
        let outcome = rand_shuffled < odds;
        if outcome {
            let won_value = (((net_bet * 256) / (odds as u128)) * game.bet_payment_adjustment)
                / FRACTIONAL_BASE;
            credits = credits + won_value;
            assert!(game.house_funds >= won_value, "{}", ERR_407);
            game.house_funds -= won_value;
        }

        account.balances.insert(&game.partner_token, &credits);
        self.internal_update_account(&account_id, &account);
        self.internal_update_game(&game_code, &game);
        self.game_count += 1;
        outcome
    }
}

// methods to be called through token receiver
impl Contract {
    pub fn user_deposit_balance(
        &mut self,
        account_id: AccountId,
        token_contract: AccountId,
        amount: u128,
    ) {
        self.assert_panic_button();

        let initial_storage = env::storage_usage();
        let mut account = self.internal_get_account(&account_id).expect(ERR_001);

        let credits = account.balances.get(&token_contract).unwrap_or(0);
        account
            .balances
            .insert(&token_contract, &(credits + amount));

        self.internal_update_account_storage_check(&account_id, account, initial_storage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    /// retrieve_credits
    /// method must:
    /// 1. Assert panic button
    /// 2. Assert enough balance
    /// 3. Remove token from user balance map
    /// 4. Send funds over with Promise (tested in integration)
    #[test]
    fn test_retrieve_credits() {
        fn closure_generator(
            token_contract_exists: bool,
            amount: u128,
            panic_button_state: bool,
            seed: u128,
        ) -> impl FnOnce() {
            move || {
                let user = format!("{}.testnet", seed);
                let context = get_context(vec![], false, 0, 1000, user.clone());
                testing_env!(context);
                let base_token = format!("{}-token.testnet", seed);
                let base_deposit = 1000;
                let other_token = format!("{}-token2.testnet", seed);
                let token_contract = if token_contract_exists {
                    base_token.clone()
                } else {
                    other_token.clone()
                };
                let mut contract = sample_contract(seed);
                contract.panic_button = panic_button_state;

                contract.internal_deposit_storage_account(&user, 1000 * ONE_NEAR);
                let mut account = contract.internal_get_account(&user).unwrap();
                account.balances.insert(&base_token, &base_deposit);
                contract.internal_update_account(&user, &account);

                contract.retrieve_credits(token_contract, U128(amount));

                assert_eq!(
                    account.balances.get(&base_token).unwrap(),
                    base_deposit - amount
                );
            }
        }

        let test_cases = [
            // 1. Assert panic button
            (true, 1, true, Some(ERR_007.to_string())),
            // 2. Assert enough balance
            (true, 1001, false, Some(ERR_401.to_string())),
            // 3. Remove token from user balance map
            (true, 900, false, None),
        ];

        let mut counter = 0;
        IntoIterator::into_iter(test_cases).for_each(|v| {
            run_test_case(closure_generator(v.0, v.1, v.2, counter), v.3);
            counter += 1;
            println!("{}", counter);
        });
    }

    /// play
    /// method must:
    /// 1. Assert panic button
    /// 2. Assert user account exists and has balance
    /// 3. Assert game exists
    /// 4. Assert bet and odds are within game limits
    /// 5. Charge all game fees
    /// 6. Increase balance of user correctly if they win
    /// 7. Return true for won games and false for lost
    #[test]
    fn test_play() {
        fn closure_generator(
            bet_size: u128,
            odds: u8,
            user_balance: u128,
            panic_button_state: bool,
            seed: u128,
        ) -> impl FnOnce() {
            move || {
                let user = format!("{}.testnet", seed);
                let context = get_context(vec![], false, 0, 1000, user.clone());
                testing_env!(context);
                let game_id = "teste.near".to_string();
                let base_token = format!("{}-token.testnet", seed);
                let partner_fee = 1000;
                let house_fee = 100;
                let owner_fee = 300;
                let nft_fee = 500;
                let max_odds = 200;
                let min_odds = 100;
                let max_bet = 100;
                let min_bet = 10;
                let bet_payment_adjustment = 10000;

                let mut contract = sample_contract(seed);
                contract.panic_button = panic_button_state;
                contract.game_count = seed;

                contract.internal_deposit_storage_account(&user, 1000 * ONE_NEAR);
                let mut account = contract.internal_get_account(&user).unwrap();
                account.balances.insert(&base_token, &user_balance);
                contract.internal_update_account(&user, &account);

                let game_settings = PartneredGame {
                    partner_owner: "anyone".to_string(),
                    blocked: false,
                    house_funds: 1_000_000,
                    partner_token: base_token.clone(),
                    partner_fee,
                    partner_balance: 0,
                    bet_payment_adjustment,
                    house_fee,
                    max_bet,
                    min_bet,
                    max_odds,
                    min_odds,
                    nft_fee,
                    owner_fee,
                };
                contract.games.insert(&game_id, &game_settings);

                let result = contract.play(
                    game_id.clone(),
                    U128(bet_size),
                    odds,
                    "_bet_type".to_string(),
                );
                let partner_fee_calc = (bet_size * partner_fee) / FRACTIONAL_BASE;
                let owner_fee_calc = (bet_size * owner_fee) / FRACTIONAL_BASE;
                let nft_fee_calc = (bet_size * nft_fee) / FRACTIONAL_BASE;
                let house_fee_calc = (bet_size * house_fee) / FRACTIONAL_BASE;

                let new_game = contract.games.get(&game_id).unwrap();
                assert_eq!(partner_fee_calc, new_game.partner_balance);

                assert_eq!(
                    owner_fee_calc,
                    contract.owner_balance.get(&base_token).unwrap()
                );
                assert_eq!(nft_fee_calc, contract.nft_balance.get(&base_token).unwrap());

                if result {
                    //user balance
                    let net_bet = bet_size
                        - partner_fee_calc
                        - owner_fee_calc
                        - nft_fee_calc
                        - house_fee_calc;
                    let won_value = (((net_bet * 256) / (odds as u128)) * bet_payment_adjustment)
                        / FRACTIONAL_BASE;
                    assert_eq!(
                        user_balance + won_value - bet_size,
                        contract
                            .internal_get_account(&user)
                            .unwrap()
                            .balances
                            .get(&base_token)
                            .unwrap()
                    );
                    // house funds
                    assert_eq!(
                        house_fee_calc + game_settings.house_funds - won_value,
                        new_game.house_funds
                    );
                } else {
                    assert_eq!(
                        house_fee_calc + game_settings.house_funds,
                        new_game.house_funds
                    );
                }
            }
        }

        let test_cases = [
            // 1. Assert panic button
            (0, 1, 0, true, Some(ERR_007.to_string())),
            // 2. Assert user account exists and has balance
            (10, 1, 5, false, Some(ERR_402.to_string())),
            // 3. Assert bet and odds are within game limits
            (10, 255, 15, false, Some(ERR_406.to_string())),
            (10, 1, 15, false, Some(ERR_405.to_string())),
            (1, 100, 15, false, Some(ERR_403.to_string())),
            (1000, 1, 1500000, false, Some(ERR_404.to_string())),
            // 5. Charge all game fees
            // 6. Increase balance of user correctly if they win
            // 7. Return true for won games and false for lost
            (100, 128, 10000, false, None),
            (100, 128, 10000, false, None),
            (100, 128, 10000, false, None),
            (100, 128, 10000, false, None),
            (100, 128, 10000, false, None),
            (100, 128, 10000, false, None),
            (100, 128, 10000, false, None),
            (100, 128, 10000, false, None),
            (100, 128, 10000, false, None),
            (100, 128, 10000, false, None),
            (100, 128, 10000, false, None),
            (100, 128, 10000, false, None),
        ];

        let mut counter = 0;
        IntoIterator::into_iter(test_cases).for_each(|v| {
            run_test_case(closure_generator(v.0, v.1, v.2, v.3, counter), v.4);
            counter += 1;
            println!("{}", counter);
        });
    }
}
