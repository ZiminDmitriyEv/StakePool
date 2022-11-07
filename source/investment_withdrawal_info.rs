use near_sdk::{AccountId, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InvestmentWithdrawalInfo {
    /// Near balance that the user requested to withdraw.
    pub near_amount: Balance,
    /// Id of account who spent funds on storage staking for this structure.
    pub account_id: AccountId
}