use near_sdk::{env, EpochHeight, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::base_error::BaseError;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DelayedWithdrawalInfo {
    yocto_near_amount: Balance,
    started_epoch_height: EpochHeight
}

impl DelayedWithdrawalInfo {
    pub fn new(yocto_near_amount: Balance, started_epoch_height: EpochHeight) -> Self {
        Self {
            yocto_near_amount,
            started_epoch_height
        }
    }
}