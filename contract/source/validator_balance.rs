use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorBalance {
    pub classic_total_near_amount: Balance,
    pub investment_total_near_amount: Balance,
    pub requested_near_amount: Balance
}