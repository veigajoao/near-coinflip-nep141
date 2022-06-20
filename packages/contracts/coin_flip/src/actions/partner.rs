use crate::*;

#[near_bindgen]
impl Contract {
    pub fn retrieve_partner_balance(&mut self, game_code: String) -> Promise {
        let mut game = self.internal_get_game(&game_code).expect(ERR_003);
        assert!(
            game.partner_owner == env::predecessor_account_id(),
            "{}",
            ERR_004
        );

        let balance = game.partner_balance;
        game.partner_balance = 0;
        self.internal_update_game(&game_code, &game);
        self.safe_transfer_project(game.partner_token, balance, game_code, game.partner_owner)
    }

    pub fn retrieve_house_funds(&mut self, game_code: String, quantity: U128) -> Promise {
        let mut game = self.internal_get_game(&game_code).expect(ERR_003);
        assert!(
            game.partner_owner == env::predecessor_account_id(),
            "{}",
            ERR_004
        );

        let balance = game.house_funds;
        assert!(balance >= quantity.0, "{}", ERR_401);

        game.house_funds -= quantity.0;
        self.internal_update_game(&game_code, &game);
        self.safe_transfer_house_funds(game.partner_token, balance, game_code, game.partner_owner)
    }
}

// methods to be called through token receiver
impl Contract {
    pub fn fund_game_house(&mut self, token_contract: AccountId, amount: u128, game_code: String) {
        let mut game = self.internal_get_game(&game_code).expect(ERR_003);
        assert_eq!(game.partner_token, token_contract, "{}", ERR_301);
        game.house_funds += amount;
        self.internal_update_game(&game_code, &game);
    }
}