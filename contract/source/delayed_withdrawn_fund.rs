use near_sdk::{Balance, AccountId, env, StorageUsage};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use super::delayed_withdrawal::DelayedWithdrawal;
use super::investment_withdrawal::InvestmentWithdrawal;
use super::MAXIMUM_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME;
use super::storage_key::StorageKey;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DelayedWithdrawnFund {
    /// Storage.
    /// AccountId - user account id.
    pub delayed_withdrawal_registry: LookupMap<AccountId, DelayedWithdrawal>,
    /// Storage
    /// AccountId - validator account id.
    /// Balance - near amount.
    pub investment_withdrawal_registry: LookupMap<AccountId, InvestmentWithdrawal>,
    /// Classic near amount needed to request from the validator.
    pub needed_to_request_classic_near_amount: Balance,
    /// Investment near amount needed to request from the validator.
    pub needed_to_request_investment_near_amount: Balance,
    /// Near balance available for withdrawal after passing the delayed withdrawal process.
    pub balance: Balance,
    /// In bytes.
    pub storage_usage_per_delayed_withdrawal: StorageUsage,
    /// In bytes.
    pub storage_usage_per_investment_withdrawal: StorageUsage
}

impl DelayedWithdrawnFund {
    pub fn new() -> Self {
        Self {
            delayed_withdrawal_registry: Self::initialize_delayed_withdrawal_registry(),
            investment_withdrawal_registry: Self::initialize_investment_withdrawal_registry(),
            needed_to_request_classic_near_amount: 0,
            needed_to_request_investment_near_amount: 0,
            balance: 0,
            storage_usage_per_delayed_withdrawal: Self::calculate_storage_usage_per_additional_delayed_withdrawal(),
            storage_usage_per_investment_withdrawal: Self::calculate_storage_usage_per_additional_investment_withdrawal()
        }
    }

    fn calculate_storage_usage_per_additional_delayed_withdrawal() -> StorageUsage {
        let mut delayed_withdrawal_registry = Self::initialize_delayed_withdrawal_registry();

        let initial_storage_usage = env::storage_usage();

        let account_id = AccountId::new_unchecked("a".repeat(MAXIMUM_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        delayed_withdrawal_registry.insert(
            &account_id,
            &DelayedWithdrawal {
                near_amount: 0,
                started_epoch_height: env::epoch_height()
            }
        );

        env::storage_usage() - initial_storage_usage
    }

    fn calculate_storage_usage_per_additional_investment_withdrawal() -> StorageUsage {
        let mut investment_withdrawal_registry = Self::initialize_investment_withdrawal_registry();

        let initial_storage_usage = env::storage_usage();

        let account_id = AccountId::new_unchecked("a".repeat(MAXIMUM_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        investment_withdrawal_registry.insert(
            &account_id,
            &InvestmentWithdrawal {
                near_amount: 0,
                account_id: account_id.clone()
            }
        );

        env::storage_usage() - initial_storage_usage
    }

    fn initialize_delayed_withdrawal_registry() -> LookupMap<AccountId, DelayedWithdrawal> {
        LookupMap::new(StorageKey::DelayedWithdrawnFund)
    }

    fn initialize_investment_withdrawal_registry() -> LookupMap<AccountId, InvestmentWithdrawal> {
        LookupMap::new(StorageKey::InvestmentWithdrawalRegisry)
    }
}