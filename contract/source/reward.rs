use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Reward {
    // Near amount received from validators as rewards in previous epoch.
    pub previous_epoch_rewards_from_validators_near_amount: Balance,
    /// Total Near amount received from validators as rewards.
    pub total_rewards_from_validators_near_amount: Balance
}