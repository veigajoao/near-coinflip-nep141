use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_contract_state(&self) -> String {
        json!(&self).to_string()
    }

    pub fn view_partner_data(&self, nft_contract: AccountId) -> PartneredGame {
        self.games.get(&nft_contract).expect(ERR_002)
    }

    pub fn get_credits(&self, token_type: AccountId, account_id: AccountId) -> U128 {
        U128(
            self.internal_get_account(&account_id)
                .expect(ERR_001)
                .balances
                .get(&token_type)
                .unwrap_or(0),
        )
    }
}