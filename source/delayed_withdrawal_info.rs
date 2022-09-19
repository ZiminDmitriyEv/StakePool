use near_sdk::{EpochHeight, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DelayedWithdrawalInfo {
    pub requested_yocto_near_amount: Balance,
    pub received_yocto_near_amount: Balance,
    pub started_epoch_height: EpochHeight
}