use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{env, AccountId, Balance, StorageUsage};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use super::account_balance::AccountBalance;
use super::get_account_id_with_maximum_length;
use super::storage_key::StorageKey;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    pub total_supply: Balance,
    /// Storage.
    /// AccountId - user account id.
    pub account_registry: LookupMap<AccountId, AccountBalance>,
    pub accounts_quantity: u64,
    pub metadata: LazyOption<FungibleTokenMetadata>,
    /// In bytes.
    pub storage_usage_per_account: StorageUsage,
}

impl FungibleToken {
    pub fn new(fungible_token_metadata: FungibleTokenMetadata) -> Self {
        Self {
            total_supply: 0,
            account_registry: Self::initialize_account_registry(),
            accounts_quantity: 0,
            metadata: Self::initialize_metadata(&fungible_token_metadata),
            storage_usage_per_account: Self::calculate_storage_usage_per_additional_account()
        }
    }

    fn calculate_storage_usage_per_additional_account() -> StorageUsage {
        let mut account_registry = Self::initialize_account_registry();

        let initial_storage_usage = env::storage_usage();

        let account_id = get_account_id_with_maximum_length();

        account_registry.insert(
            &account_id,
            &AccountBalance {
                token_amount: 0,
                classic_near_amount: 0,
                investment_near_amount: 0
            }
        );

        env::storage_usage() - initial_storage_usage
    }

    fn initialize_account_registry() -> LookupMap<AccountId, AccountBalance> {
        LookupMap::new(StorageKey::FungibleToken)
    }

    fn initialize_metadata(fungible_token_metadata: &FungibleTokenMetadata) -> LazyOption<FungibleTokenMetadata> {
        LazyOption::new(StorageKey::FungibleTokenMetadata, Some(fungible_token_metadata))
    }
}