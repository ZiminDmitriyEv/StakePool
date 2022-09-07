use crate::ONE_TERA;
use near_sdk::{env, StorageUsage, AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use super::base_error::BaseError;
use super::delayed_unstake_validator_group::DelayedUnstakeValidatorGroup;
use super::storage_key::StorageKey;
use super::validator_info::ValidatorInfo;
use super::validator_staking_contract_version::ValidatorStakingContractVersion;
use super::MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatingNode {
    /// Must be changed each epoch to the next value.
    pub current_delayed_unstake_validator_group: DelayedUnstakeValidatorGroup,
    pub validator_account_registry: UnorderedMap<AccountId, ValidatorInfo>,
    pub validator_accounts_quantity: u64,
    pub validator_accounts_maximum_quantity: Option<u64>,
    pub quantity_of_validators_accounts_updated_in_current_epoch: u64,
    /// In bytes.
    pub storage_usage_per_validator_account: StorageUsage,
}

impl ValidatingNode {
    /// In fact it is needed 10 Tgas, but this is with a margin of safety.
    const DEPOSIT_AND_STAKE_TGAS: u64 = 15;

    pub fn new(validators_maximum_quantity: Option<u64>) -> Result<Self, BaseError> {
        Ok(
            Self {
                current_delayed_unstake_validator_group: DelayedUnstakeValidatorGroup::First,
                validator_account_registry: Self::initialize_validator_account_registry(),
                validator_accounts_quantity: 0,
                validator_accounts_maximum_quantity: validators_maximum_quantity,
                quantity_of_validators_accounts_updated_in_current_epoch: 0,
                storage_usage_per_validator_account: Self::calculate_storage_usage_per_additional_validator_account()?
            }
        )
    }

    fn calculate_storage_usage_per_additional_validator_account() -> Result<StorageUsage, BaseError> {
        let mut validator_account_registry = Self::initialize_validator_account_registry();

        let initial_storage_usage = env::storage_usage();

        let account_id = AccountId::new_unchecked("a".repeat(MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        validator_account_registry.insert(
            &account_id, &ValidatorInfo::new(ValidatorStakingContractVersion::Classic, DelayedUnstakeValidatorGroup::First)
        );

        if env::storage_usage() < initial_storage_usage {
            return Err(BaseError::Logic);
        }

        Ok(env::storage_usage() - initial_storage_usage)
    }

    fn initialize_validator_account_registry() -> UnorderedMap<AccountId, ValidatorInfo> {
        UnorderedMap::new(StorageKey::ValidatorAccountRegistry)
    }
}