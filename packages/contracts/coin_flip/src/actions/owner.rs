use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn emergency_panic(&mut self) -> bool {
        self.only_owner();
        self.panic_button = !self.panic_button;
        self.panic_button
    }

    #[payable]
    pub fn update_contract(&mut self, new_owner: AccountId) {
        self.only_owner();

        self.owner_id = new_owner;
    }

    #[payable]
    pub fn retrieve_owner_funds(&mut self, token_index: u64) -> Promise {
        assert_one_yocto();
        let key = self.owner_balance.keys_as_vector().get(token_index).expect(ERR_204);
        let value = self.owner_balance.values_as_vector().get(token_index).unwrap();
        assert!(value > 0, "{}", ERR_203);
        self.owner_balance.insert(&key, &0);
        self.safe_transfer_owner(key, value)
    }

    #[payable]
    pub fn retrieve_nft_funds(&mut self, token_index: u64) -> Promise {
        assert_one_yocto();
        let key = self.nft_balance.keys_as_vector().get(token_index).expect(ERR_204);
        let value = self.nft_balance.values_as_vector().get(token_index).unwrap();
        assert!(value > 0, "{}", ERR_203);
        self.nft_balance.insert(&key, &0);
        self.safe_transfer_nft(key, value)
    }

    //create new partnered game
    #[payable]
    pub fn create_new_partner(
        &mut self,
        partner_owner: AccountId,
        nft_contract: AccountId,
        token_contract: AccountId,
        partner_fee: U128,
        bet_payment_adjustment: U128,
        house_fee: U128,
        max_bet: U128,
        min_bet: U128,
        max_odds: u8,
        min_odds: u8,
        nft_fee: U128,
        owner_fee: U128,
    ) {
        self.only_owner();
        assert!(!self.games.contains_key(&nft_contract), "{}", ERR_003);
        let contract_id = env::current_account_id();
        let mut contract_account = self.internal_get_account(&contract_id).unwrap();
        let initial_storage = env::storage_usage();

        self.nft_balance.insert(&token_contract, &0);
        self.owner_balance.insert(&token_contract, &0);

        assert!(max_bet.0 > min_bet.0, "{}", ERR_206);
        assert!(max_odds > min_odds, "{}", ERR_206);
        assert!(house_fee.0 <= FRACTIONAL_BASE, "{}", ERR_205);
        assert!(partner_fee.0 <= FRACTIONAL_BASE, "{}", ERR_205);
        assert!(nft_fee.0 <= FRACTIONAL_BASE, "{}", ERR_205);
        assert!(owner_fee.0 <= FRACTIONAL_BASE, "{}", ERR_205);
        assert!(bet_payment_adjustment.0 <= FRACTIONAL_BASE, "{}", ERR_205);

        let game_settings = PartneredGame {
            partner_owner,
            blocked: false,
            house_funds: 0,
            partner_token: token_contract,
            partner_fee: partner_fee.0,
            partner_balance: 0,

            bet_payment_adjustment: bet_payment_adjustment.0,
            house_fee: house_fee.0,
            max_bet: max_bet.0,
            min_bet: min_bet.0,
            max_odds,
            min_odds,
            nft_fee: nft_fee.0,
            owner_fee: owner_fee.0
        };
        self.games.insert(&nft_contract, &game_settings);

        contract_account.track_storage_usage(initial_storage);
        self.internal_update_account(&contract_id, &contract_account);
    }

    #[payable]
    pub fn alter_partner(
        &mut self,
        game_id: String,
        partner_owner: AccountId,
        partner_fee: U128,
        blocked: bool,
        bet_payment_adjustment: U128,
        house_fee: U128,
        max_bet: U128,
        min_bet: U128,
        max_odds: u8,
        min_odds: u8,
        nft_fee: U128,
        owner_fee: U128,
    ) {
        self.only_owner();
        assert!(self.games.contains_key(&game_id), "{}", ERR_002);
        assert!(max_bet.0 > min_bet.0, "{}", ERR_206);
        assert!(max_odds > min_odds, "{}", ERR_206);
        assert!(house_fee.0 <= FRACTIONAL_BASE, "{}", ERR_205);
        assert!(partner_fee.0 <= FRACTIONAL_BASE, "{}", ERR_205);
        assert!(nft_fee.0 <= FRACTIONAL_BASE, "{}", ERR_205);
        assert!(owner_fee.0 <= FRACTIONAL_BASE, "{}", ERR_205);
        assert!(bet_payment_adjustment.0 <= FRACTIONAL_BASE, "{}", ERR_205);
        let mut game = self.internal_get_game(&game_id).expect(ERR_002);
        game.partner_owner = partner_owner;
        game.partner_fee = partner_fee.0;
        game.blocked = blocked;
        game.bet_payment_adjustment = bet_payment_adjustment.0;
        game.house_fee = house_fee.0;
        game.max_bet = max_bet.0;
        game.min_bet = min_bet.0;
        game.max_odds = max_odds;
        game.min_odds = min_odds;
        game.nft_fee = nft_fee.0;
        game.owner_fee = owner_fee.0;
        self.internal_update_game(&game_id, &game);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    /// emergency_panic
    /// Method must:
    /// 1. Assert caller is owner
    /// 2. Assert caller deposited 1 yoctoNear
    /// 3. Toggle the bool variable panic_button
    ///    in the contract global
    #[test]
    fn test_emergency_panic() {
        fn closure_generator(
            signer: AccountId,
            deposit: u128,
            panic_button_state: bool,
            seed: u128,
        ) -> impl FnOnce() {
            move || {
                let context = get_context(vec![], false, deposit, 0, signer);
                testing_env!(context);
                // instantiate a contract variable with the counter at zero
                let mut contract = sample_contract(seed);
                contract.panic_button = panic_button_state;

                contract.emergency_panic();

                assert_ne!(panic_button_state, contract.panic_button);
            }
        }

        let test_cases = [
            // 1. Assert caller is owner
            (
                SIGNER_ACCOUNT.to_string(),
                1,
                false,
                Some(ERR_006.to_string()),
            ),
            // 2. Assert caller deposited 1 yoctoNear
            (
                OWNER_ACCOUNT.to_string(),
                0,
                false,
                Some("Requires attached deposit of exactly 1 yoctoNEAR".to_string()),
            ),
            // 3. Toggle the bool variable panic_button
            //    in the contract global
            (OWNER_ACCOUNT.to_string(), 1, false, None),
        ];

        let mut counter = 0;
        IntoIterator::into_iter(test_cases).for_each(|v| {
            run_test_case(closure_generator(v.0, v.1, v.2, counter), v.3);
            counter += 1;
            println!("{}", counter);
        });
    }

    /// update_contract
    /// Method must:
    /// 1. Assert caller is owner
    /// 2. Assert caller deposited 1 yoctoNear
    /// 3. Alter owner_id variable in the contract
    ///    to new owner
    #[test]
    fn test_update_contract() {
        fn closure_generator(
            signer: AccountId,
            deposit: u128,
            panic_button_state: bool,
            seed: u128,
        ) -> impl FnOnce() {
            move || {
                let context = get_context(vec![], false, deposit, 0, signer);
                testing_env!(context);
                // instantiate a contract variable with the counter at zero
                let mut contract = sample_contract(seed);
                contract.panic_button = panic_button_state;

                contract.update_contract(SIGNER_ACCOUNT.to_string());

                assert_eq!(contract.owner_id, SIGNER_ACCOUNT.to_string());
            }
        }

        let test_cases = [
            // 1. Assert caller is owner
            (
                SIGNER_ACCOUNT.to_string(),
                1,
                false,
                Some(ERR_006.to_string()),
            ),
            // 2. Assert caller deposited 1 yoctoNear
            (
                OWNER_ACCOUNT.to_string(),
                0,
                false,
                Some("Requires attached deposit of exactly 1 yoctoNEAR".to_string()),
            ),
            // 3. Toggle the bool variable panic_button
            //    in the contract global
            (OWNER_ACCOUNT.to_string(), 1, false, None),
        ];

        let mut counter = 0;
        IntoIterator::into_iter(test_cases).for_each(|v| {
            run_test_case(closure_generator(v.0, v.1, v.2, counter), v.3);
            counter += 1;
            println!("{}", counter);
        });
    }

    /// retrieve_owner_funds
    /// Method must:
    /// 1. Assert that caller deposits one yoctoNear
    /// 2. Assert that there is a balance to withdraw
    /// 3. Assert that token exists
    /// 4. Withdraw the entirety of this balance
    /// 5. Send promise to transfer token
    #[test]
    fn test_retrieve_owner_funds() {

        fn closure_generator(
            deposit: u128,
            token_index: u64,
            token_balance: u128,
            seed: u128,
        ) -> impl FnOnce() {
            move || {
                let token = format!("{}.testnet", seed);
                let context = get_context(vec![], false, deposit, 1, SIGNER_ACCOUNT.to_string());
                testing_env!(context);
                // instantiate a contract variable with the counter at zero
                let mut contract = sample_contract(seed);
                contract.owner_balance.insert(&token, &token_balance);

                contract.retrieve_owner_funds(token_index);

                assert_eq!(contract.owner_balance.get(&token).unwrap(), 0);

            }
        }

        let test_cases = [
            // 1. Assert that caller deposits one yoctoNear
            (
                0,
                0,
                0,
                Some("Requires attached deposit of exactly 1 yoctoNEAR".to_string()),
            ),
            // 2. Assert that there is a balance to withdraw
            (
                1,
                0,
                0,
                Some(ERR_203.to_string()),
            ),
            // 3. Assert that token exists
            (
                1,
                4,
                0,
                Some(ERR_204.to_string()),
            ),
            // 4. Withdraw the entirety of this balance
            // 5. Send promise to transfer token
            (1, 0, 1000, None),
        ];

        let mut counter = 0;
        IntoIterator::into_iter(test_cases).for_each(|v| {
            run_test_case(closure_generator(v.0, v.1, v.2, counter), v.3);
            counter += 1;

        });
    }

    /// retrieve_nft_funds
    /// Method must:
    /// 1. Assert that caller deposits one yoctoNear
    /// 2. Assert that there is a balance to withdraw
    /// 3. Assert that token exists
    /// 4. Withdraw the entirety of this balance
    /// 5. Send promise to transfer token
    #[test]
    fn test_retrieve_nft_funds() {

        fn closure_generator(
            deposit: u128,
            token_index: u64,
            token_balance: u128,
            seed: u128,
        ) -> impl FnOnce() {
            move || {
                let token = format!("{}.testnet", seed);
                let context = get_context(vec![], false, deposit, 1, SIGNER_ACCOUNT.to_string());
                testing_env!(context);
                // instantiate a contract variable with the counter at zero
                let mut contract = sample_contract(seed);
                contract.nft_balance.insert(&token, &token_balance);

                contract.retrieve_nft_funds(token_index);

                assert_eq!(contract.nft_balance.get(&token).unwrap(), 0);

            }
        }

        let test_cases = [
            // 1. Assert that caller deposits one yoctoNear
            (
                0,
                0,
                0,
                Some("Requires attached deposit of exactly 1 yoctoNEAR".to_string()),
            ),
            // 2. Assert that there is a balance to withdraw
            (
                1,
                0,
                0,
                Some(ERR_203.to_string()),
            ),
            // 3. Assert that token exists
            (
                1,
                4,
                0,
                Some(ERR_204.to_string()),
            ),
            // 4. Withdraw the entirety of this balance
            // 5. Send promise to transfer token
            (1, 0, 1000, None),
        ];

        let mut counter = 0;
        IntoIterator::into_iter(test_cases).for_each(|v| {
            run_test_case(closure_generator(v.0, v.1, v.2, counter), v.3);
            counter += 1;

        });
    }

    /// create_new_partner
    /// Method must:
    /// 1. Assert caller is owner
    /// 2. Assert that caller deposits one yoctoNear
    /// 3. Assert that contract has storage paid for new game
    /// 4. Assert data validations
    ///    a. max_bet > min_bet
    ///    b. max_odds > min_odds
    ///    c. fees <= FRACTION_BASE 
    /// 5. Insert new game into LookupMap
    #[test]
    fn test_create_new_partner() {

        fn closure_generator(
            signer: AccountId,
            deposit: u128,
            contract_storage_balance: u128,
            params: (U128, U128, U128, U128, U128, u8, u8, U128, U128),
            seed: u128,
        ) -> impl FnOnce() {
            move || {
                let partner_owner = format!("{}-partner.testnet", seed);
                let nft_contract = format!("{}-nft.testnet", seed);
                let token = format!("{}.testnet", seed);
                let context = get_context(vec![], false, deposit, 1_000 * ONE_NEAR, signer);
                testing_env!(context);
                // instantiate a contract variable with the counter at zero
                let mut contract = sample_contract(seed);
                contract.internal_deposit_storage_account(&CONTRACT_ACCOUNT.to_string(), contract_storage_balance);

                assert!(!contract.games.contains_key(&nft_contract));

                contract.create_new_partner(
                    partner_owner.clone(),
                    nft_contract.clone(),
                    token.clone(),
                    params.0,
                    params.1,
                    params.2,
                    params.3,
                    params.4,
                    params.5,
                    params.6,
                    params.7,
                    params.8,
                );

                assert!(contract.games.contains_key(&nft_contract));

            }
        }

        let test_cases = [
            // 1. Assert caller is owner
            (
                SIGNER_ACCOUNT.to_string(),
                1,
                0,
                (U128(0), U128(0), U128(0), U128(0), U128(0), 0, 0, U128(0), U128(0)),
                Some(ERR_006.to_string()),
            ),
            // 2. Assert that caller deposits one yoctoNear
            (
                OWNER_ACCOUNT.to_string(),
                0,
                0,
                (U128(0), U128(0), U128(0), U128(0), U128(0), 0, 0, U128(0), U128(0)),
                Some("Requires attached deposit of exactly 1 yoctoNEAR".to_string()),
            ),
            // 3. Assert that contract has storage paid for new game
            (
                OWNER_ACCOUNT.to_string(),
                1,
                0,
                (U128(0), U128(0), U128(0), U128(100), U128(10), 2, 1, U128(0), U128(0)),
                Some(ERR_101.to_string()),
            ),
            // 4. Assert data validations
            //    a. max_bet > min_bet
            (
                OWNER_ACCOUNT.to_string(),
                1,
                0,
                (U128(0), U128(0), U128(0), U128(100), U128(101), 2, 1, U128(0), U128(0)),
                Some(ERR_206.to_string()),
            ),
            //    b. max_odds > min_odds
            (
                OWNER_ACCOUNT.to_string(),
                1,
                0,
                (U128(0), U128(0), U128(0), U128(100), U128(10), 2, 3, U128(0), U128(0)),
                Some(ERR_206.to_string()),
            ),
            //    c. fees <= FRACTION_BASE 
            (
                OWNER_ACCOUNT.to_string(),
                1,
                1_000 * ONE_NEAR,
                (U128(FRACTIONAL_BASE + 1), U128(0), U128(0), U128(100), U128(10), 2, 1, U128(0), U128(0)),
                Some(ERR_205.to_string()),
            ),
            (
                OWNER_ACCOUNT.to_string(),
                1,
                1_000 * ONE_NEAR,
                (U128(0), U128(FRACTIONAL_BASE + 1), U128(0), U128(100), U128(10), 2, 1, U128(0), U128(0)),
                Some(ERR_205.to_string()),
            ),
            (
                OWNER_ACCOUNT.to_string(),
                1,
                1_000 * ONE_NEAR,
                (U128(0), U128(0), U128(FRACTIONAL_BASE + 1), U128(100), U128(10), 2, 1, U128(0), U128(0)),
                Some(ERR_205.to_string()),
            ),
            (
                OWNER_ACCOUNT.to_string(),
                1,
                1_000 * ONE_NEAR,
                (U128(0), U128(0), U128(0), U128(100), U128(10), 2, 1, U128(0), U128(FRACTIONAL_BASE + 1)),
                Some(ERR_205.to_string()),
            ),
            (
                OWNER_ACCOUNT.to_string(),
                1,
                1_000 * ONE_NEAR,
                (U128(0), U128(0), U128(0), U128(100), U128(10), 2, 1, U128(FRACTIONAL_BASE + 1), U128(0)),
                Some(ERR_205.to_string()),
            ),
            // 5. Insert new game into LookupMap
            (
                OWNER_ACCOUNT.to_string(),
                1,
                1_000 * ONE_NEAR,
                (U128(0), U128(0), U128(0), U128(100), U128(10), 2, 1, U128(0), U128(0)),
                None,
            ),
        ];

        let mut counter = 0;
        IntoIterator::into_iter(test_cases).for_each(|v| {
            run_test_case(closure_generator(v.0, v.1, v.2, v.3, counter), v.4);
            counter += 1;

        });
    }

    /// alter_partner
    /// Method must:
    /// 1. Assert caller is owner
    /// 2. Assert that caller deposits one yoctoNear
    /// 3. Assert that game exists
    /// 4. Assert data validations
    ///    a. max_bet > min_bet
    ///    b. max_odds > min_odds
    ///    c. fees <= FRACTION_BASE 
    /// 5. Insert new game into LookupMap
    #[test]
    fn test_alter_partner() {

        fn closure_generator(
            signer: AccountId,
            deposit: u128,
            params: (Option<String>, U128, bool, U128, U128, U128, U128, u8, u8, U128, U128),
            seed: u128,
        ) -> impl FnOnce() {
            move || {
                let partner_owner = format!("{}-partner.testnet", seed);
                let nft_contract = format!("{}-nft.testnet", seed);
                let token = format!("{}.testnet", seed);
                let context = get_context(vec![], false, deposit, 1_000 * ONE_NEAR, signer);
                testing_env!(context);
                // instantiate a contract variable with the counter at zero
                let mut contract = sample_contract(seed);
                // contract.internal_deposit_storage_account(&CONTRACT_ACCOUNT.to_string(), contract_storage_balance);

                let game_settings = PartneredGame {
                    partner_owner: partner_owner.clone(),
                    blocked: false,
                    house_funds: 0,
                    partner_token: token,
                    partner_fee: 0,
                    partner_balance: 0,
        
                    bet_payment_adjustment: 10_000,
                    house_fee: 0,
                    max_bet: 0,
                    min_bet: 0,
                    max_odds: 200,
                    min_odds: 10,
                    nft_fee: 0,
                    owner_fee: 0
                };
                contract.games.insert(&nft_contract, &game_settings);

                let call_game;
                if let Some(v) = params.0 {
                    call_game = v;
                } else {
                    call_game = nft_contract.clone();
                }
                contract.alter_partner(
                    call_game,
                    partner_owner.clone(),
                    params.1,
                    params.2,
                    params.3,
                    params.4,
                    params.5,
                    params.6,
                    params.7,
                    params.8,
                    params.9,
                    params.10
                );

                assert!(contract.games.contains_key(&nft_contract));

            }
        }

        let test_cases = [
            // 1. Assert caller is owner
            (
                SIGNER_ACCOUNT.to_string(),
                1,
                (None, U128(0), false, U128(0), U128(0), U128(0), U128(0), 0, 0, U128(0), U128(0)),
                Some(ERR_006.to_string()),
            ),
            // 2. Assert that caller deposits one yoctoNear
            (
                OWNER_ACCOUNT.to_string(),
                0,
                (None, U128(0), false, U128(0), U128(0), U128(0), U128(0), 0, 0, U128(0), U128(0)),
                Some("Requires attached deposit of exactly 1 yoctoNEAR".to_string()),
            ),
            // 3. Assert that game exists
            (
                OWNER_ACCOUNT.to_string(),
                1,
                (Some("Other".to_string()), U128(0), false, U128(0), U128(0), U128(100), U128(10), 2, 1, U128(0), U128(0)),
                Some(ERR_002.to_string()),
            ),
            // 4. Assert data validations
            //    a. max_bet > min_bet
            (
                OWNER_ACCOUNT.to_string(),
                1,
                (None, U128(0), false, U128(0), U128(0), U128(100), U128(101), 2, 1, U128(0), U128(0)),
                Some(ERR_206.to_string()),
            ),
            //    b. max_odds > min_odds
            (
                OWNER_ACCOUNT.to_string(),
                1,
                (None, U128(0), false, U128(0), U128(0), U128(100), U128(10), 2, 3, U128(0), U128(0)),
                Some(ERR_206.to_string()),
            ),
            //    c. fees <= FRACTION_BASE 
            (
                OWNER_ACCOUNT.to_string(),
                1,
                (None, U128(FRACTIONAL_BASE + 1), false, U128(0), U128(0), U128(100), U128(10), 2, 1, U128(0), U128(0)),
                Some(ERR_205.to_string()),
            ),
            (
                OWNER_ACCOUNT.to_string(),
                1,
                (None, U128(0), false, U128(FRACTIONAL_BASE + 1), U128(0), U128(100), U128(10), 2, 1, U128(0), U128(0)),
                Some(ERR_205.to_string()),
            ),
            (
                OWNER_ACCOUNT.to_string(),
                1,
                (None, U128(0), false, U128(0), U128(FRACTIONAL_BASE + 1), U128(100), U128(10), 2, 1, U128(0), U128(0)),
                Some(ERR_205.to_string()),
            ),
            (
                OWNER_ACCOUNT.to_string(),
                1,
                (None, U128(0), false, U128(0), U128(0), U128(100), U128(10), 2, 1, U128(0), U128(FRACTIONAL_BASE + 1)),
                Some(ERR_205.to_string()),
            ),
            (
                OWNER_ACCOUNT.to_string(),
                1,
                (None, U128(0), false, U128(0), U128(0), U128(100), U128(10), 2, 1, U128(FRACTIONAL_BASE + 1), U128(0)),
                Some(ERR_205.to_string()),
            ),
            // 5. Insert new game into LookupMap
            (
                OWNER_ACCOUNT.to_string(),
                1,
                (None, U128(0), false, U128(0), U128(0), U128(100), U128(10), 2, 1, U128(0), U128(0)),
                None,
            ),
        ];

        let mut counter = 0;
        IntoIterator::into_iter(test_cases).for_each(|v| {
            run_test_case(closure_generator(v.0, v.1, v.2, counter), v.3);
            counter += 1;

        });
    }

}
