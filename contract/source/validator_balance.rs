use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorBalance {
    /// Classic part of valdator total Near amount.
    pub classic_near_amount: Balance,
    /// Investment part of valdator total Near amount.
    pub investment_near_amount: Balance,
    /// Requested to withdrawal Near amount.
    pub requested_to_withdrawal_near_amount: Balance
}

impl ValidatorBalance {
    pub fn get_staked_balance(&self) -> Balance {
        self.classic_near_amount + self.investment_near_amount
    }
}