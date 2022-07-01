use near_sdk::{env, Balance, PublicKey, StorageUsage, Promise, AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap};
use super::base_error::BaseError;
use super::storage_key::StorageKey;
use super::validator_info::ValidatorInfo;
use super::validator_staking_contract_version::ValidatorStakingContractVersion;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatingNode {
    validator_account_registry: UnorderedMap<AccountId, ValidatorInfo>,
    validator_accounts_quantity: u64,
    validator_accounts_maximum_quantity: Option<u64>,
    storage_usage_per_validator_account: StorageUsage,
}

impl ValidatingNode {
    const MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME: u8 = 64;        // TODO такое же уже есть. Вынести все в один файл для констант?

    pub fn new(validators_maximum_quantity: Option<u64>) -> Result<Self, BaseError> {
        Ok(
            Self {
                validator_account_registry: Self::initialize_validator_account_registry(),
                validator_accounts_quantity: 0,
                validator_accounts_maximum_quantity: validators_maximum_quantity,
                storage_usage_per_validator_account: Self::calculate_storage_usage_per_additional_validator_account()?
            }
        )
    }

    pub fn register_validator_account(
        &mut self, account_id: &AccountId, staking_contract_version: ValidatorStakingContractVersion
    ) -> Result<(), BaseError> {
        if let Some(maximium_quantity) = self.validator_accounts_maximum_quantity {
            if self.validator_accounts_quantity >= maximium_quantity {
                return Err(BaseError::ValidatorAccountsMaximumQuantityExceeding);
            }
        }

        if let Some(_) = self.validator_account_registry.insert(account_id, &ValidatorInfo::new(staking_contract_version)) {
            return Err(BaseError::ValidatorAccountIsAlreadyRegistered);
        }
        self.validator_accounts_quantity = self.validator_accounts_quantity + 1;

        Ok(())
    }

    pub fn unregister_validator_account(&mut self, account_id: &AccountId) -> Result<(), BaseError> {
        if let None = self.validator_account_registry.remove(account_id) {
            return Err(BaseError::ValidatorAccountIsNotRegistered);
        }
        if self.validator_accounts_quantity == 0 {
            return Err(BaseError::Logic);
        }

    // TODO проверить, есть ли на валидаторе деньги. если нет, то можно
        self.validator_accounts_quantity = self.validator_accounts_quantity - 1;

        Ok(())
    }

    pub fn distribute_available_for_staking_balance(&mut self, yocto_near_amount: Balance) -> Result<Promise, BaseError> {
        if self.validator_accounts_quantity == 0 {
            return Err(BaseError::ValidatorAccountsZeroQuantity)
        }
// TODO TODO Вот здесь нуно распределить все без остатка
// Поставить проверки, чтоюы никуда не зашел ноль !!!!!!!!!!!!!!!!!!!!!!!
        let yocto_near_amount_for_one_validator = yocto_near_amount / (self.validator_accounts_quantity as u128);

        let mut validator_account_registry: Vec<(PublicKey, ValidatorInfo)> = vec![];



        todo!();
        // for (account_id, mut validator_info) in self.validator_account_registry.iter() {

        //     validator_info.increase_staked_balance(yocto_near_amount_for_one_validator)?;

        //     validator_account_registry.push((account_id, validator_info));
        // }
        
        // match promise {
        //     Some(promise_) => {
        //         Ok(promise_)
        //     }
        //     None => {
        //         return Err(BaseError::Logic);
        //     }
        // }
    }

    pub fn get_storage_staking_price_per_additional_validator_account(&self) -> Result<Balance, BaseError> {
        match Balance::from(self.storage_usage_per_validator_account)
            .checked_mul(env::storage_byte_cost()) {
            Some(value) => {
                Ok(value)
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        }
    }

    fn calculate_storage_usage_per_additional_validator_account() -> Result<StorageUsage, BaseError> {
        let mut validator_account_registry = Self::initialize_validator_account_registry();
    
        let initial_storage_usage = env::storage_usage();
    
        let account_id = AccountId::new_unchecked("a".repeat(Self::MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));
    
        validator_account_registry.insert(&account_id, &ValidatorInfo::new(ValidatorStakingContractVersion::Classic));

        if env::storage_usage() < initial_storage_usage {
            return Err(BaseError::Logic);
        }
    
        Ok(env::storage_usage() - initial_storage_usage)
    }

    fn initialize_validator_account_registry() -> UnorderedMap<AccountId, ValidatorInfo> {
        UnorderedMap::new(StorageKey::ValidatorAccountRegistry)
    }
}