use near_sdk::{env, AccountId, Balance, StorageUsage};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use super::base_error::BaseError;
use super::fungible_token_metadata::FungibleTokenMetadata;
use super::storage_key::StorageKey;
use super::MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    pub owner_id: AccountId,
    pub total_supply: Balance,
    pub token_account_registry: LookupMap<AccountId, Balance>,
    pub token_accounts_quantity: u64,
    pub token_metadata: LazyOption<FungibleTokenMetadata>,
        /// In bytes.
    pub storage_usage_per_token_account: StorageUsage,
}

impl FungibleToken {
    pub fn new(owner_id: AccountId) -> Result<Self, BaseError> {
        let fungible_token_metadata = FungibleTokenMetadata::new();
        if !fungible_token_metadata.is_valid() {
            return Err(BaseError::InvalidFungibleTokenMetadata);
        }

        Ok(
            Self {
                owner_id,
                total_supply: 0,
                token_account_registry: Self::initialize_token_account_registry(),
                token_accounts_quantity: 0,
                token_metadata: Self::initialize_fungible_token_metadata(&fungible_token_metadata),
                storage_usage_per_token_account: Self::calculate_storage_usage_per_additional_token_account()?
            }
        )
    }

    fn calculate_storage_usage_per_additional_token_account() -> Result<StorageUsage, BaseError> {
        let mut token_account_registry = Self::initialize_token_account_registry();

        let initial_storage_usage = env::storage_usage();

        let account_id = AccountId::new_unchecked("a".repeat(MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        token_account_registry.insert(&account_id, &0);

        if env::storage_usage() < initial_storage_usage {
            return Err(BaseError::Logic);
        }

        Ok(env::storage_usage() - initial_storage_usage)
    }

    fn initialize_token_account_registry() -> LookupMap<AccountId, Balance> {
        LookupMap::new(StorageKey::FungibleToken)
    }

    fn initialize_fungible_token_metadata(fungible_token_metadata: &FungibleTokenMetadata) -> LazyOption<FungibleTokenMetadata> {
        LazyOption::new(StorageKey::FungibleTokenMetadata, Some(fungible_token_metadata))
    }
}