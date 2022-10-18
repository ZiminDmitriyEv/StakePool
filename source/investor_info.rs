use near_sdk::{Balance, AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use super::storage_key::StorageKey;
use super::base_error::BaseError;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InvestorInfo {
    pub validator_distribution_account_registry: UnorderedMap<AccountId, Balance>, // При таком Мэпе нужно ограничить количество валидаторов здесь или же писать клиент  // TODO Название. Почему везде в таких случаях пишется _account_. Стоит ли менять?
    pub investment_balance: Balance,
}

impl InvestorInfo {
    pub fn new(investor_account_id: AccountId) -> Result<Self, BaseError> {
        Ok(
            Self {
                validator_distribution_account_registry: Self::initialize_validator_distribution_account_registry(investor_account_id),
                investment_balance: 0
            }
        )
    }

    pub fn initialize_validator_distribution_account_registry(investor_account_id: AccountId) -> UnorderedMap<AccountId, Balance> {
        UnorderedMap::new(StorageKey::ValidatorDistributionAccountRegistry { investor_account_id })
    }
}