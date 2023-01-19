use near_sdk::AccountId;
use near_sdk::borsh::{self, BorshSerialize};
use near_sdk::BorshStorageKey;

/// Do not change the order of variants.
/// The number of options must be less than or equal to 256 (1 byte).
#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    InvestorInvestment,
    FungibleToken,
    FungibleTokenMetadata,
    Validator,
    DelayedWithdrawnFund,
    Distribution {
        investor_account_id: AccountId
    },
    InvestmentWithdrawal
}