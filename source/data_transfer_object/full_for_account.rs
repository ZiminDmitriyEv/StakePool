use near_sdk::serde::{Deserialize, Serialize};
use super::account_balance::AccountBalance;
use super::delayed_withdrawal_details::DelayedWithdrawalDetails;
use super::full::Full;
use super::storage_staking_requested_coverage::StorageStakingRequestedCoverage;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FullForAccount {
    pub full: Full,
    pub account_balance: AccountBalance,
    pub delayed_withdrawal_details: Option<DelayedWithdrawalDetails>,
    pub storage_staking_requested_coverage: StorageStakingRequestedCoverage
}