use crate::ONE_TERA;
use near_sdk::{env, near_bindgen, Balance, PublicKey, StorageUsage, Promise, AccountId, PromiseResult, Gas, EpochHeight};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap};
use super::base_error::BaseError;
use super::delayed_unstake_validator_group::DelayedUnstakeValidatorGroup;
use super::stake_pool::StakePool;
use super::stake_pool::StakePoolExt;
use super::storage_key::StorageKey;
use super::validator_info::ValidatorInfo;
use super::validator_staking_contract_version::ValidatorStakingContractVersion;
use super::xcc_staking_pool::ext_staking_pool;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatingNode {
    /// Must be changed each epoch to the next value.
    current_delayed_unstake_validator_group: DelayedUnstakeValidatorGroup,
    validator_account_registry: UnorderedMap<AccountId, ValidatorInfo>,
    validator_accounts_quantity: u64,
    validator_accounts_maximum_quantity: Option<u64>,
    /// In bytes.
    storage_usage_per_validator_account: StorageUsage,
    is_stake_distributed_in_current_epoch: bool
}

impl ValidatingNode {
    /// In fact it is needed 10 Tgas, but this is with a margin of safety.
    const DEPOSIT_AND_STAKE_TGAS: u64 = 15;
    const MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME: u8 = 64;        // TODO такое же уже есть. Вынести все в один файл для констант?

    pub fn new(validators_maximum_quantity: Option<u64>) -> Result<Self, BaseError> {
        Ok(
            Self {
                current_delayed_unstake_validator_group: DelayedUnstakeValidatorGroup::First,
                validator_account_registry: Self::initialize_validator_account_registry(),
                validator_accounts_quantity: 0,
                validator_accounts_maximum_quantity: validators_maximum_quantity,
                storage_usage_per_validator_account: Self::calculate_storage_usage_per_additional_validator_account()?,
                is_stake_distributed_in_current_epoch: false
            }
        )
    }

    pub fn register_validator_account(
        &mut self, account_id: &AccountId,
        staking_contract_version: ValidatorStakingContractVersion,
        delayed_unstake_validator_group: DelayedUnstakeValidatorGroup
    ) -> Result<(), BaseError> {
        if let Some(maximium_quantity) = self.validator_accounts_maximum_quantity {
            if self.validator_accounts_quantity >= maximium_quantity {
                return Err(BaseError::ValidatorAccountsMaximumQuantityExceeding);
            }
        }

        if let Some(_) = self.validator_account_registry.insert(
            account_id, &ValidatorInfo::new(staking_contract_version, delayed_unstake_validator_group)
        ) {
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

    pub fn increase_validator_stake(    // TODO Пока это делает на итерациях с клиента, могут сделать депозит или снять наоборот
        &mut self, validator_account_id: &AccountId, yocto_near_amount: Balance
    ) -> Result<Promise, BaseError> {     // TODO какое минимально значение для дистрибуции.? Нужно ли регестрировать аккаунт на стороне стэкеинг-пуул?
        if self.validator_accounts_quantity == 0 {
            return Err(BaseError::ValidatorAccountsZeroQuantity)
        }

        // let deposit_and_stake_gas = Gas(ONE_TERA * Self::DEPOSIT_AND_STAKE_TGAS);           // TODO проверка, сколько газа прикрепили

        match self.validator_account_registry.get(validator_account_id) {
            Some(validator_info) => {
                match *validator_info.get_staking_contract_version() {
                    ValidatorStakingContractVersion::Classic => {
                        return Ok(
                            ext_staking_pool::ext(validator_account_id.clone())
                                .with_attached_deposit(yocto_near_amount)
                                // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                                .deposit_and_stake()
                                .then(
                                    StakePool::ext(env::current_account_id())           // TODO TODO TODO TODO  смотреть, сколько на коллбек Газа.
                                        .increase_validator_stake_callback(
                                            &validator_account_id, yocto_near_amount, env::epoch_height()
                                        )
                                )
                            );
                    }
                }
            }
            None => {
                return Err(BaseError::ValidatorAccountIsNotRegistered);
            }
        }
    }

    pub fn update_after_increase_validator_stake(                         // TODO  Как здесь назвать или что сделать, есть много подобных названий
        &mut self, validator_account_id: &AccountId, staked_balance: Balance, current_epoch_height: EpochHeight
    ) -> Result<(), BaseError> {
        match self.validator_account_registry.get(validator_account_id) {
            Some(mut validator_info) => {
                validator_info.increase_staked_balance(staked_balance)?;
                validator_info.set_last_stake_increasing_epoch_height(current_epoch_height);

                self.validator_account_registry.insert(validator_account_id, &validator_info);

                Ok(())
            }
            None => {
                return Err(BaseError::ValidatorAccountIsNotRegistered);
            }
        }
    }

    pub fn update_validator_info(  // TODO ЧТо будет, если валидатор перестал работать, что придет с контракта. Не прервется ли из-за этго цепочка выполнения апдейтов
        &mut self, validator_account_id: &AccountId
    ) -> Result<Promise, BaseError> {
        match self.validator_account_registry.get(validator_account_id) {
            Some(validator_info) => {
                let current_epoch_haight = env::epoch_height();

                if validator_info.get_last_update_epoch_haight() < current_epoch_haight {
                    match *validator_info.get_staking_contract_version() {
                        ValidatorStakingContractVersion::Classic => {
                            return Ok(
                                ext_staking_pool::ext(validator_account_id.clone())
                                    // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                                    .get_account_total_balance(env::current_account_id())
                                    .then(
                                        StakePool::ext(env::current_account_id())           // TODO TODO TODO TODO  смотреть, сколько на коллбек Газа.
                                            .update_validator_info_callback(&validator_account_id, current_epoch_haight)
                                    )
                                );
                        }
                    }
                }

                return Err(BaseError::ValidatorInfoAlreadyUpdated);
            }
            None => {
                return Err(BaseError::ValidatorAccountIsNotRegistered);
            }
        }
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