use near_sdk::{env, AccountId, Balance, StorageUsage};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use super::base_error::BaseError;
use super::fungible_token_metadata::FungibleTokenMetadata;
use super::storage_key::StorageKey;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    owner_id: AccountId,
    total_supply: Balance,
    token_account_registry: LookupMap<AccountId, Balance>,
    token_accounts_quantity: u64,

    /// In bytes
    storage_usage_per_token_account: StorageUsage,
    token_metadata: LazyOption<FungibleTokenMetadata>,
}

impl FungibleToken {                                        // TODO стоит ли продублировать зашиту в методах
    const MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME: u8 = 64;

    const TOKEN_IMAGE_PATH: &'static str = "todo_path";
    const TOKEN_NAME:  &'static str = "todo_name";
    const TOKEN_SYMPOL:  &'static str = "todo_symbol";
    const TOKEN_DECIMALS: u8 = 24;

    pub fn new(owner_id: AccountId) -> Result<Self, BaseError> {
        let fungible_token_metadata = FungibleTokenMetadata::new(
            Self::TOKEN_NAME.to_string(),
            Self::TOKEN_SYMPOL.to_string(),
            Some(Self::TOKEN_IMAGE_PATH.to_string()),
            None,                                            // TODO TODO TODO TODO TODO что это 
            None,                                       // TODO TODO TODO TODO TODO что это
            Self::TOKEN_DECIMALS
        );
        if !fungible_token_metadata.is_valid() {
            return Err(BaseError::InvalidFungibleTokenMetadata);
        }

        Ok(
            Self { 
                owner_id,
                total_supply: 0,
                token_account_registry: Self::initialize_token_account_registry_lookup_map(),
                token_accounts_quantity: 0,
                storage_usage_per_token_account: Self::calculate_storage_usage_per_additional_token_account()?,
                token_metadata: Self::initialize_fungible_token_metadata_lazy_option(&fungible_token_metadata)
            }
        )
    }

    pub fn register_token_account(&mut self, account_id: &AccountId) -> Result<(), BaseError> {
        if let Some(_) = self.token_account_registry.insert(account_id, &0) {
            return Err(BaseError::TokenAccountAlreadyRegistered);
        }
        self.token_accounts_quantity = self.token_accounts_quantity + 1;

        Ok(())
    }

    pub fn unregister_token_account(&mut self, account_id: &AccountId) -> Result<(), BaseError> {
        match self.token_account_registry.remove(account_id) {
            Some(yocto_token_balance) => {
                if yocto_token_balance != 0 {
                    return Err(BaseError::UnregisterTokenAccountWithNonZeroTokenBalance);
                }
            }
            None => {
                return Err(BaseError::TokenAccountIsNotRegistered);
            }
        }
        self.token_accounts_quantity = self.token_accounts_quantity - 1;

        Ok(())
    }

    pub fn increase_token_account_balance(&mut self, account_id: &AccountId, yocto_token_amount: Balance) -> Result<(), BaseError> {
        if yocto_token_amount == 0 {
            return Err(BaseError::ZeroIncreasing);
        }

        match self.token_account_registry.get(account_id) {
            Some(yocto_token_balance) => {
                match yocto_token_balance
                    .checked_add(yocto_token_amount) {
                    Some(yocto_token_balance_) => {
                        self.token_account_registry.insert(account_id, &yocto_token_balance_);
                        self.total_supply = match self.total_supply
                            .checked_add(yocto_token_amount) {
                            Some(total_supply_) => {
                                total_supply_
                            }
                            None => {
                                return Err(BaseError::CalculationOwerflow);
                            }
                        };

                        return Ok(());
                    }
                    None => {
                        return Err(BaseError::CalculationOwerflow);
                    }
                }
            },
            None => {
                return Err(BaseError::TokenAccountIsNotRegistered);
            }
        }
    }

    pub fn decrease_token_account_balance(&mut self, account_id: &AccountId, yocto_token_amount: Balance) -> Result<(), BaseError> {
        if yocto_token_amount == 0 {
            return Err(BaseError::ZeroDecreasing);
        }

        match self.token_account_registry.get(account_id) {
            Some(yocto_token_balance) => {
                match yocto_token_balance
                    .checked_sub(yocto_token_amount) {
                    Some(yocto_token_balance_) => {
                        self.token_account_registry.insert(account_id, &yocto_token_balance_);
                        self.total_supply = match self.total_supply
                            .checked_sub(yocto_token_amount) {
                            Some(total_supply_) => {
                                total_supply_
                            }
                            None => {
                                return Err(BaseError::CalculationOwerflow);
                            }
                        };

                        return Ok(());
                    }
                    None => {
                        return Err(BaseError::InsufficientTokenAccountBalance);
                    }
                }
            },
            None => {
                return Err(BaseError::TokenAccountIsNotRegistered);
            }
        }
    }

    pub fn get_storage_staking_price_per_additional_token_account(&self) -> Result<Balance, BaseError> {
        match Balance::from(self.storage_usage_per_token_account).checked_mul(env::storage_byte_cost()) {
            Some(value) => {
                Ok(value)
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        }
    }

    pub fn is_token_account_registered(&self, account_id: &AccountId) -> bool {
        self.token_account_registry.contains_key(account_id)
    }

    pub fn can_unregister_token_account(&self, account_id: &AccountId) -> Result<bool, BaseError> {
        match self.token_account_registry.get(account_id) {
            Some(yocto_token_balance) => {
                Ok(yocto_token_balance == 0)
            },
            None => {
                return Err(BaseError::TokenAccountIsNotRegistered);
            }
        }
    }

    pub fn get_token_account_balance(&self, account_id: &AccountId) -> Result<Balance, BaseError> {
        match self.token_account_registry.get(account_id) {
            Some(yocto_token_balance) => {
                Ok(yocto_token_balance)
            },
            None => {
                return Err(BaseError::TokenAccountIsNotRegistered);
            }
        }
    }

    pub fn get_total_token_supply(&self) -> Balance {
        self.total_supply
    }

    pub fn get_token_accounts_quantity(&self) -> u64 {
        self.token_accounts_quantity
    }

    fn calculate_storage_usage_per_additional_token_account() -> Result<StorageUsage, BaseError> {
        let mut token_account_registry = Self::initialize_token_account_registry_lookup_map();

        let initial_storage_usage = env::storage_usage();

        let account_id = AccountId::new_unchecked("a".repeat(Self::MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        token_account_registry.insert(&account_id, &0);

        if env::storage_usage() < initial_storage_usage {
            return Err(BaseError::Logic);
        }

        Ok(env::storage_usage() - initial_storage_usage)
    }

    fn initialize_token_account_registry_lookup_map() -> LookupMap<AccountId, Balance> {
        LookupMap::new(StorageKey::FungibleToken1)
    }

    fn initialize_fungible_token_metadata_lazy_option(fungible_token_metadata: &FungibleTokenMetadata) -> LazyOption<FungibleTokenMetadata> {
        LazyOption::new(StorageKey::FungibleTokenMetadata1, Some(fungible_token_metadata))
    }
}