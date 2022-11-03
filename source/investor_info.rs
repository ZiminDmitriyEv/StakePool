use near_sdk::{Balance, AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use super::storage_key::StorageKey;
use super::base_error::BaseError;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InvestorInfo {
    pub distribution_registry: LookupMap<AccountId, Balance>,
    pub distributions_quantity: u64,      // TODO Нужно ли.
    pub staked_balance: Balance
}

impl InvestorInfo {
    pub fn new(investor_account_id: AccountId) -> Result<Self, BaseError> {
        Ok(
            Self {
                distribution_registry: Self::initialize_distribution_registry(investor_account_id),
                distributions_quantity: 0,
                staked_balance: 0
            }
        )
    }

    pub fn initialize_distribution_registry(investor_account_id: AccountId) -> LookupMap<AccountId, Balance> {
        LookupMap::new(StorageKey::DistributionRegistry { investor_account_id })
    }
}