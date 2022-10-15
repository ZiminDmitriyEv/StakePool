use crate::ONE_TERA;
use near_sdk::{env, StorageUsage, AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use super::base_error::BaseError;
use super::MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME;
use super::storage_key::StorageKey;
use super::validator_info::ValidatorInfo;
use super::validator_staking_contract_version::ValidatorStakingContractVersion;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatingNode {
    pub validator_account_registry: UnorderedMap<AccountId, ValidatorInfo>,
    /// Registry of investors who are allowed to make an investment deposit.
    pub investor_account_registry: UnorderedSet<AccountId>,
    pub validator_accounts_quantity: u64,
    pub validator_accounts_maximum_quantity: Option<u64>,
    pub preffered_validtor_account: Option<AccountId>,
    pub quantity_of_validators_accounts_updated_in_current_epoch: u64,
    /// In bytes.
    pub storage_usage_per_validator_account: StorageUsage,
    /// In bytes.
    pub storage_usage_per_investor_account: StorageUsage
}

impl ValidatingNode {
    /// In fact it is needed 10 Tgas, but this is with a margin of safety.
    const DEPOSIT_AND_STAKE_TGAS: u64 = 15;

    pub fn new(validators_maximum_quantity: Option<u64>) -> Result<Self, BaseError> {
        Ok(
            Self {
                validator_account_registry: Self::initialize_validator_account_registry(),
                investor_account_registry: Self::initialize_investor_account_registry(),
                validator_accounts_quantity: 0,
                validator_accounts_maximum_quantity: validators_maximum_quantity,
                preffered_validtor_account: None,
                quantity_of_validators_accounts_updated_in_current_epoch: 0,
                storage_usage_per_validator_account: Self::calculate_storage_usage_per_additional_validator_account()?,
                storage_usage_per_investor_account: Self::calculate_storage_usage_per_additional_investor_account()?
            }
        )
    }

    fn calculate_storage_usage_per_additional_validator_account() -> Result<StorageUsage, BaseError> {
        let mut validator_account_registry = Self::initialize_validator_account_registry();

        let initial_storage_usage = env::storage_usage();

        let account_id = AccountId::new_unchecked("a".repeat(MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        validator_account_registry.insert(
            &account_id, &ValidatorInfo::new(ValidatorStakingContractVersion::Classic)
        );

        if env::storage_usage() < initial_storage_usage {
            return Err(BaseError::Logic);
        }

        Ok(env::storage_usage() - initial_storage_usage)
    }

    fn calculate_storage_usage_per_additional_investor_account() -> Result<StorageUsage, BaseError> {
        let mut investor_account_registry = Self::initialize_investor_account_registry();

        let initial_storage_usage = env::storage_usage();

        let account_id = AccountId::new_unchecked("a".repeat(MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        investor_account_registry.insert(&account_id);

        if env::storage_usage() < initial_storage_usage {
            return Err(BaseError::Logic);
        }

        Ok(env::storage_usage() - initial_storage_usage)
    }

    fn initialize_validator_account_registry() -> UnorderedMap<AccountId, ValidatorInfo> {
        UnorderedMap::new(StorageKey::ValidatorAccountRegistry)
    }

    fn initialize_investor_account_registry() -> UnorderedSet<AccountId> {
        UnorderedSet::new(StorageKey::InvestorAccountRegistry)
    }
}