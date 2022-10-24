use near_sdk::{Balance, AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use super::storage_key::StorageKey;
use super::base_error::BaseError;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InvestorInfo {       // TODO можно сделать через ЛукапМэп и каутера, так как аккаунт будет удаляться при снятии
    pub validator_distribution_account_registry: LookupMap<AccountId, Balance>,     // TODO Название. Почему везде в таких случаях пишется _account_. Стоит ли менять?
    pub validator_distribution_accounts_quantity: u64,
    pub staked_balance: Balance     // TODO название
}

impl InvestorInfo {
    pub fn new(investor_account_id: AccountId) -> Result<Self, BaseError> {
        Ok(
            Self {
                validator_distribution_account_registry: Self::initialize_validator_distribution_account_registry(investor_account_id),
                validator_distribution_accounts_quantity: 0,
                staked_balance: 0
            }
        )
    }

    pub fn initialize_validator_distribution_account_registry(investor_account_id: AccountId) -> LookupMap<AccountId, Balance> {
        LookupMap::new(StorageKey::ValidatorDistributionAccountRegistry { investor_account_id })
    }
}