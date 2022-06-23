use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap};
use near_sdk::{env, Balance, PublicKey, StorageUsage, Promise};
use super::base_error::BaseError;
use super::storage_key::StorageKey;
use super::validator_info::ValidatorInfo;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatingNode {
    validator_account_registry: LazyOption<UnorderedMap<PublicKey, ValidatorInfo>>,
    validator_accounts_quantity: u64,
    validator_accounts_maximum_quantity: Option<u64>,
    storage_usage_per_validator_account: StorageUsage,
}

impl ValidatingNode {
    pub fn new(validators_maximum_quantity: Option<u64>) -> Result<Self, BaseError> {
        Ok(
            Self {
                validator_account_registry: Self::initialize_validator_account_registry_lazy_option(),
                validator_accounts_quantity: 0,
                validator_accounts_maximum_quantity: validators_maximum_quantity,
                storage_usage_per_validator_account: Self::calculate_storage_usage_per_additional_validator_account()?
            }
        )
    }

    pub fn register_validator_account(&mut self, stake_public_key: &PublicKey) -> Result<(), BaseError> {
        match self.validator_account_registry.get() {
            Some(mut validator_account_registry_) => {
                if let Some(maximium_quantity) = self.validator_accounts_maximum_quantity {
                    if self.validator_accounts_quantity >= maximium_quantity {
                        return Err(BaseError::ValidatorAccountsMaximumQuantityExceeding);
                    }
                }

                if let Some(_) = validator_account_registry_.insert(stake_public_key, &ValidatorInfo::new()) {
                    return Err(BaseError::ValidatorAccountIsAlreadyRegistered);
                }
                self.validator_account_registry.set(&validator_account_registry_);
                self.validator_accounts_quantity = self.validator_accounts_quantity + 1;

                Ok(())
            },
            None => {
                return Err(BaseError::Logic);
            }
        }
    }

    pub fn unregister_validator_account(&mut self, stake_public_key: &PublicKey) -> Result<(), BaseError> {
        match self.validator_account_registry.get() {
            Some(mut validator_account_registry_) => {
                if let None = validator_account_registry_.remove(stake_public_key) {
                    return Err(BaseError::ValidatorAccountIsNotRegistered);
                }
                if self.validator_accounts_quantity == 0 {
                    return Err(BaseError::Logic);
                }

            // TODO проверить, есть ли на валидаторе деньги. если нет, то можно

                self.validator_account_registry.set(&validator_account_registry_);
                self.validator_accounts_quantity = self.validator_accounts_quantity - 1;

                Ok(())
            },
            None => {
                return Err(BaseError::Logic);
            }
        }
    }

    pub fn distribute_available_for_staking_balance(&mut self, yocto_near_amount: Balance) -> Result<Promise, BaseError> {
        if self.validator_accounts_quantity == 0 {
            return Err(BaseError::ValidatorAccountsZeroQuantity)
        }
// TODO TODO Вот здесь нуно распределить все без остатка
// Поставить проверки, чтоюы никуда не зашел ноль !!!!!!!!!!!!!!!!!!!!!!!
        let yocto_near_amount_for_one_validator = yocto_near_amount / (self.validator_accounts_quantity as u128);

        let mut validator_account_registry: Vec<(PublicKey, ValidatorInfo)> = vec![];

        let mut promise: Option<Promise> = None;
        match self.validator_account_registry.get() {
            Some(validator_account_registry_) => {
                for (stake_public_key, mut validator_info) in validator_account_registry_.iter() {
                    match promise {
                        Some(promise_) => {
                            promise = Some (
                                promise_.then(
                                    Promise::new(env::current_account_id())
                                        .stake(yocto_near_amount_for_one_validator, stake_public_key.clone())
                                )
                            );
                        },
                        None => {
                            promise = Some(
                                Promise::new(env::current_account_id())
                                    .stake(yocto_near_amount_for_one_validator, stake_public_key.clone())
                            );
                        }
                    }

                    validator_info.increase_staked_balance(yocto_near_amount_for_one_validator)?;

                    validator_account_registry.push((stake_public_key, validator_info));
                }
            },
            None => {
                return Err(BaseError::Logic);
            }
        }
        // self.validator_account_registry.set(&validator_account_registry);
        
        match promise {
            Some(promise_) => {
                Ok(promise_)
            }
            None => {
                return Err(BaseError::Logic);
            }
        }
    }

    pub fn get_storage_staking_price_per_additional_validator_account(&self) -> Result<Balance, BaseError> {
        match Balance::from(self.storage_usage_per_validator_account).checked_mul(env::storage_byte_cost()) {
            Some(value) => {
                Ok(value)
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        }
    }

    fn calculate_storage_usage_per_additional_validator_account() -> Result<StorageUsage, BaseError> {
        let validator_account_registry = Self::initialize_validator_account_registry_lazy_option();
    
        let initial_storage_usage = env::storage_usage();
    
        // The longest key format.
        let public_key: PublicKey = "secp256k1:qMoRgcoXai4mBPsdbHi1wfyxF9TdbPCF4qSDQTRP3TfescSRoUdSx6nmeQoN3aiwGzwMyGXAb1gUjBTv5AY8DXj"
            .parse()
            .unwrap();          // TODO TODO УБарть Анвреп
    
        match validator_account_registry.get() {
            Some(ref mut validator_account_registry_) => {
                validator_account_registry_.insert(&public_key, &ValidatorInfo::new());

                if env::storage_usage() < initial_storage_usage {
                    return Err(BaseError::Logic);
                }
            
                Ok(env::storage_usage() - initial_storage_usage)
            },
            None => {
                return Err(BaseError::Logic);
            }
        }
    }

    fn initialize_validator_account_registry_lazy_option() -> LazyOption<UnorderedMap<PublicKey, ValidatorInfo>> {
        LazyOption::new(StorageKey::ValidatorNode1, Some(&UnorderedMap::new(StorageKey::ValidatorNode2)))
    }
}