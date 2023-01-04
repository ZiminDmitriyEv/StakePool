use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Reward {
    pub previous_epoch_rewards_from_validators_near_amount: Balance,
    pub total_rewards_from_validators_near_amount: Balance
}