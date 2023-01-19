use near_sdk::{Balance, AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use super::storage_key::StorageKey;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InvestorInvestment {
    /// Storage
    /// AccountId - validator account id.
    /// Balance - staked Near Amount.
    pub distribution_registry: LookupMap<AccountId, Balance>,
    pub distributions_quantity: u64,
    /// Total Near amount distributed on validators.
    pub staked_balance: Balance
}

impl InvestorInvestment {
    pub fn new(investor_account_id: AccountId) -> Self {
        Self {
            distribution_registry: Self::initialize_distribution_registry(investor_account_id),
            distributions_quantity: 0,
            staked_balance: 0
        }
    }

    pub fn initialize_distribution_registry(investor_account_id: AccountId) -> LookupMap<AccountId, Balance> {
        LookupMap::new(StorageKey::Distribution { investor_account_id })
    }
}