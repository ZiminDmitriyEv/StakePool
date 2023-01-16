use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorBalance {
    pub classic_staked_balance: Balance,
    pub investment_staked_balance: Balance,
    pub unstaked_balance: Balance,
    pub requested_to_withdrawal_unstaked_balance: Balance
}