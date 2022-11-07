use near_sdk::{EpochHeight, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DelayedWithdrawalInfo {
    /// Near balance that the user requested to withdraw.
    pub near_amount: Balance,
    /// It is only needed in order to understand when it is possible to give
    /// the user his funds, because the funds can only be returned after 8 epochs
    /// with delayed_withdraw method.
    pub started_epoch_height: EpochHeight
}