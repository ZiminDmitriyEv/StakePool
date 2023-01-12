use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountBalance {
    pub token_amount: Balance,
    /// Amount of funds that remained as a result of the conversion at the exchange rate.
    pub near_amount: Balance
}