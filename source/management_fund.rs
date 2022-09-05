use near_sdk::{Balance, AccountId, env, StorageUsage};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use super::base_error::BaseError;
use super::delayed_withdrawal_info::DelayedWithdrawalInfo;
use super::storage_key::StorageKey;
use super::MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ManagementFund {
    pub available_for_staking_balance: Balance,
    staked_balance: Balance,
    delayed_withdrawal_account_registry: UnorderedMap<AccountId, DelayedWithdrawalInfo>,
    is_distributed_on_validators_in_current_epoch: bool,
    /// In bytes.
    storage_usage_per_delayed_withdrawal_account: StorageUsage,
}

impl ManagementFund {
    pub fn new() -> Result<Self, BaseError> {
        Ok(
            Self {
                available_for_staking_balance: 0,
                staked_balance: 0,
                delayed_withdrawal_account_registry: Self::initialize_delayed_withdrawal_account_registry(),
                is_distributed_on_validators_in_current_epoch: false,
                storage_usage_per_delayed_withdrawal_account: Self::calculate_storage_usage_per_additional_delayed_withdrawal_account()?
            }
        )
    }

    pub fn increase_available_for_staking_balance(&mut self, yocto_near_amount: Balance) -> Result<(), BaseError> {
        self.available_for_staking_balance = match self.available_for_staking_balance
            .checked_add(yocto_near_amount) {
            Some(available_for_staking_balance_) => {
                available_for_staking_balance_
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        };

        Ok(())
    }

    pub fn decrease_available_for_staking_balance(&mut self, yocto_near_amount: Balance) -> Result<(), BaseError> {
        self.available_for_staking_balance = match self.available_for_staking_balance
            .checked_sub(yocto_near_amount) {
            Some(available_for_staking_balance_) => {
                available_for_staking_balance_
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        };

        Ok(())
    }

    pub fn increase_staked_balance(&mut self, yocto_near_amount: Balance) -> Result<(), BaseError> {
        self.staked_balance = match self.staked_balance
            .checked_add(yocto_near_amount) {
            Some(staked_balance_) => {
                staked_balance_
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        };

        Ok(())
    }

    pub fn decrease_staked_balance(&mut self, yocto_near_amount: Balance) -> Result<(), BaseError> {
        self.staked_balance = match self.staked_balance
            .checked_sub(yocto_near_amount) {
            Some(staked_balance_) => {
                staked_balance_
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        };

        Ok(())
    }

    pub fn set_is_distributed_on_validators_in_current_epoch(&mut self, is_distributed_on_validators_in_current_epoch: bool) {
        self.is_distributed_on_validators_in_current_epoch = is_distributed_on_validators_in_current_epoch;
    }

    pub fn get_staked_balance(&self) -> Balance {
        self.staked_balance
    }

    pub fn get_is_distributed_on_validators_in_current_epoch(&self) -> bool {
        self.is_distributed_on_validators_in_current_epoch
    }

    pub fn get_management_fund_amount(&self) -> Result<Balance, BaseError> {
        match self.available_for_staking_balance
            .checked_add(self.staked_balance) {
            Some(management_fund_amount) => {
                Ok(management_fund_amount)
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        }
    }

    fn calculate_storage_usage_per_additional_delayed_withdrawal_account() -> Result<StorageUsage, BaseError> {
        let mut delayed_withdrawal_account_registry = Self::initialize_delayed_withdrawal_account_registry();

        let initial_storage_usage = env::storage_usage();

        let account_id = AccountId::new_unchecked("a".repeat(MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        delayed_withdrawal_account_registry.insert(
            &account_id, &DelayedWithdrawalInfo { yocto_near_amount: 0, started_epoch_height: env::epoch_height() }
        );

        if env::storage_usage() < initial_storage_usage {
            return Err(BaseError::Logic);
        }

        Ok(env::storage_usage() - initial_storage_usage)
    }

    fn initialize_delayed_withdrawal_account_registry() -> UnorderedMap<AccountId, DelayedWithdrawalInfo> {
        UnorderedMap::new(StorageKey::DelayedWithdrawalAccountRegistry)
    }
}