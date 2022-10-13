use near_sdk::{Balance, AccountId, env, StorageUsage};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use super::base_error::BaseError;
use super::delayed_withdrawal_info::DelayedWithdrawalInfo;
use super::MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME;
use super::storage_key::StorageKey;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ManagementFund {
    pub classic_unstaked_balance: Balance,
    pub classic_staked_balance: Balance,
    pub investament_unstaked_balance: Balance,
    pub investament_staked_balance: Balance,
    pub delayed_withdrawal_account_registry: UnorderedMap<AccountId, DelayedWithdrawalInfo>,
    pub delayed_withdrawal_balance: Balance,
    pub is_distributed_on_validators_in_current_epoch: bool,
    /// In bytes.
    pub storage_usage_per_delayed_withdrawal_account: StorageUsage,
}

impl ManagementFund {
    pub fn new() -> Result<Self, BaseError> {
        Ok(
            Self {
                classic_unstaked_balance: 0,
                classic_staked_balance: 0,
                investament_unstaked_balance: 0,
                investament_staked_balance: 0,
                delayed_withdrawal_account_registry: Self::initialize_delayed_withdrawal_account_registry(),
                delayed_withdrawal_balance: 0,
                is_distributed_on_validators_in_current_epoch: false,
                storage_usage_per_delayed_withdrawal_account: Self::calculate_storage_usage_per_additional_delayed_withdrawal_account()?
            }
        )
    }

    pub fn get_management_fund_amount(&self) -> Balance {
        self.classic_unstaked_balance
        + self.classic_staked_balance
        + self.investament_unstaked_balance
        + self.investament_unstaked_balance
    }

    fn calculate_storage_usage_per_additional_delayed_withdrawal_account() -> Result<StorageUsage, BaseError> {
        let mut delayed_withdrawal_account_registry = Self::initialize_delayed_withdrawal_account_registry();

        let initial_storage_usage = env::storage_usage();

        let account_id = AccountId::new_unchecked("a".repeat(MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        delayed_withdrawal_account_registry.insert(
            &account_id,
            &DelayedWithdrawalInfo {
                requested_near_amount: 0,
                received_near_amount: 0,
                started_epoch_height: env::epoch_height()
            }
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