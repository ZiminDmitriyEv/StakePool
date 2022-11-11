use near_sdk::{Balance, AccountId, env, StorageUsage};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use super::delayed_withdrawal_info::DelayedWithdrawalInfo;
use super::investment_withdrawal_info::InvestmentWithdrawalInfo;
use super::MAXIMUM_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME;
use super::storage_key::StorageKey;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DelayedWithdrawnFund {
    /// Storage.
    /// AccountId - user account id.
    pub account_registry: LookupMap<AccountId, DelayedWithdrawalInfo>,
    /// Storage
    /// AccountId - validator account id.
    /// Balance - near amount.
    pub investment_withdrawal_registry: LookupMap<AccountId, InvestmentWithdrawalInfo>,
    /// Classic near amount to be requested from the validator.
    pub needed_to_request_classic_near_amount: Balance,
    /// Investment near amount to be requested from the validator.
    pub needed_to_request_investment_near_amount: Balance,                                          // TODO вынести в отдельную структуру с реестрои выше?
    /// Near balance available for withdrawal after passing the delayed withdrawal process.
    pub balance: Balance,          // TODO посмотреть в свойствах и в методах, стоит ли именить near_balance на balance и подобное, то есть, near_ уже может быть в контексте.
    /// In bytes.
    pub storage_usage_per_account: StorageUsage,
    /// In bytes.
    pub storage_usage_per_investment_withdrawal: StorageUsage
}

impl DelayedWithdrawnFund {
    pub fn new() -> Self {
        Self {
            account_registry: Self::initialize_account_registry(),
            investment_withdrawal_registry: Self::initialize_investment_withdrawal_registry(),
            needed_to_request_classic_near_amount: 0,
            needed_to_request_investment_near_amount: 0,
            balance: 0,
            storage_usage_per_account: Self::calculate_storage_usage_per_additional_account(),
            storage_usage_per_investment_withdrawal: Self::calculate_storage_usage_per_additional_investment_withdrawal()
        }
    }

    fn calculate_storage_usage_per_additional_account() -> StorageUsage {
        let mut account_registry = Self::initialize_account_registry();

        let initial_storage_usage = env::storage_usage();

        let account_id = AccountId::new_unchecked("a".repeat(MAXIMUM_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        account_registry.insert(
            &account_id,
            &DelayedWithdrawalInfo {
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
            &InvestmentWithdrawalInfo {
                near_amount: 0,
                account_id: account_id.clone()
            }
        );

        env::storage_usage() - initial_storage_usage
    }

    fn initialize_account_registry() -> LookupMap<AccountId, DelayedWithdrawalInfo> {
        LookupMap::new(StorageKey::DelayedWithdrawnFund)
    }

    fn initialize_investment_withdrawal_registry() -> LookupMap<AccountId, InvestmentWithdrawalInfo> {
        LookupMap::new(StorageKey::InvestmentWithdrawalRegisry)
    }
}