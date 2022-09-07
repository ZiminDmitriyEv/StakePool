use near_sdk::{env, EpochHeight, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DelayedWithdrawalInfo {
    pub yocto_near_amount: Balance,
    pub started_epoch_height: EpochHeight
}