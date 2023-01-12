use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountBalance {
    pub token_amount: Balance,
    /// Amount of classic Near that remained as a result of the conversion at the exchange rate.
    pub classic_near_amount: Balance,
    /// Amount of investment Near that remained as a result of the conversion at the exchange rate.
    pub investment_near_amount: Balance
}