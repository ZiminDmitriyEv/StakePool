use core::convert::Into;
use near_sdk::{env, near_bindgen, PanicOnDefault, AccountId, Balance, EpochHeight, Promise, PromiseResult, StorageUsage};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use super::aggregated_information::AggregatedInformation;
use super::base_error::BaseError;
use super::delayed_unstake_validator_group::DelayedUnstakeValidatorGroup;
use super::fee_registry::FeeRegistry;
use super::fee::Fee;
use super::fungible_token::FungibleToken;
use super::management_fund::ManagementFund;
use super::validating_node::ValidatingNode;
use super::validator_info_dto::ValidatorInfoDto;
use super::validator_info::ValidatorInfo;
use super::validator_staking_contract_version::ValidatorStakingContractVersion;
use super::xcc_staking_pool::ext_staking_pool;
use uint::construct_uint;

construct_uint! {
    pub struct U256(4);
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]     // TODO проверить все типы данных. LazyOption, например, добавить. !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!1
pub struct StakePool {      // TODO Можно перенести Структуру в место, где будут все структуры. А Функциональность оставить. Раз я решил делать Публичные поля
    owner_id: AccountId,
    manager_id: AccountId,
    rewards_receiver_account_id: AccountId,
    everstake_rewards_receiver_account_id: AccountId,
    fungible_token: FungibleToken,
    management_fund: ManagementFund,
    fee_registry: FeeRegistry,                          // TODO сделать через Next epoch.
    validating_node: ValidatingNode,
    current_epoch_height: EpochHeight,
    previous_epoch_rewards_from_validators_yocto_near_amount: Balance,       // TODO МОЖет, сделать через ПрошлыйКурс?
    total_rewards_from_validators_yocto_near_amount: Balance        // TODO Все, что связано с ревардс, перенести в структуру?
}

impl StakePool {        // TODO TODO TODO добавить логи к каждой манипуляции с деньгами или event. Интерфейсы
    fn internal_new(
        manager_id: Option<AccountId>,
        rewards_receiver_account_id: AccountId,
        everstake_rewards_receiver_account_id: AccountId,
        rewards_fee: Option<Fee>,
        everstake_rewards_fee: Option<Fee>,
        validators_maximum_quantity: Option<u64>
    ) -> Result<Self, BaseError> {
        if env::state_exists() {
            return Err(BaseError::ContractStateAlreadyInitialized);
        }

        if let Some(ref rewards_fee_) = rewards_fee {
            rewards_fee_.assert_valid()?;
        }
        if let Some(ref everstake_rewards_fee_) = everstake_rewards_fee {
            everstake_rewards_fee_.assert_valid()?;
        }

        let manager_id_ = match manager_id {
            Some(manager_id__) => {
                manager_id__
            }
            None => {
                env::predecessor_account_id()
            }
        };

        // TODO rewards_receiver_account_id != everstake_rewards_receiver_account_id
        // TODO Взять деньги (зарезервировать) для регистрации этих двуз аккаунтов lido_rewards_receiver_account_id,
                // everstake_rewards_receiver_account_id,
                // !!!!!!!!!!!!!!
        // TODO ЗАрегистрировать эти два токен аккаунта, и не удалять их, если с них снимаются в ноль. ОБРАТИТЬ ВНИМАНИЕ, ЧТО СНЯТИЕ в НОЛЬ ВЛЕЕТ удаление. А они не должны быть удалены

        Ok(
            Self {
                owner_id: env::predecessor_account_id(),
                manager_id: manager_id_,
                rewards_receiver_account_id,
                everstake_rewards_receiver_account_id,
                fee_registry: FeeRegistry { rewards_fee, everstake_rewards_fee },
                fungible_token: FungibleToken::new(env::predecessor_account_id())?,
                management_fund: ManagementFund::new()?,
                validating_node: ValidatingNode::new(validators_maximum_quantity)?,
                current_epoch_height: env::epoch_height(),
                previous_epoch_rewards_from_validators_yocto_near_amount: 0,
                total_rewards_from_validators_yocto_near_amount: 0
            }
        )
    }

    fn internal_deposit(&mut self) -> Result<(), BaseError> {       // TODO TODO TODO TODO TODO Нужно ли делать так, чтобы еслм  is_distributed_on_validators_in_current_epoch, то кладем сразу на Префферед валидатор
        self.assert_epoch_is_synchronized()?;

        let account_id = env::predecessor_account_id();

        let mut yocto_near_amount = env::attached_deposit();
        let mut yocto_token_balance: Balance = match self.fungible_token.token_account_registry.get(&account_id) {
            Some(yocto_token_balance_) => yocto_token_balance_,
            None => {
                let storage_staking_price_per_additional_token_account = Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_token_account)?;
                if yocto_near_amount < storage_staking_price_per_additional_token_account {
                    return Err(BaseError::InsufficientNearDepositForStorageStaking);
                }
                yocto_near_amount -= storage_staking_price_per_additional_token_account;

                self.fungible_token.token_accounts_quantity += 1;

                0
            }
        };
        if yocto_near_amount == 0 {
            return Err(BaseError::InsufficientNearDeposit);
        }

        let yocto_token_amount = self.convert_yocto_near_amount_to_yocto_token_amount(yocto_near_amount)?;
        if yocto_token_amount == 0 {
            return Err(BaseError::InsufficientNearDeposit);
        }

        yocto_token_balance += yocto_token_amount;

        self.management_fund.available_for_staking_balance += yocto_near_amount;
        self.fungible_token.total_supply += yocto_token_amount;
        self.fungible_token.token_account_registry.insert(&account_id, &yocto_token_balance);

        Ok(())
    }

    fn internal_instant_withdraw(&mut self, yocto_token_amount: u128) -> Result<Promise, BaseError> {   // TODO проставить процент на снятие!!
        self.assert_epoch_is_synchronized()?;

        let account_id = env::predecessor_account_id();

        if yocto_token_amount == 0 {
            return Err(BaseError::InsufficientTokenDeposit);
        }

        let mut yocto_near_amount = self.convert_yocto_token_amount_to_yocto_near_amount(yocto_token_amount)?;
        if yocto_near_amount == 0 {
            return Err(BaseError::InsufficientTokenDeposit);
        }
        if yocto_near_amount > self.management_fund.available_for_staking_balance {
            return Err(BaseError::InsufficientAvailableForStakingBalance);
        }

        let mut yocto_token_balance = match self.fungible_token.token_account_registry.get(&account_id) {
            Some(yocto_token_balance_) => yocto_token_balance_,
            None => {
                return Err(BaseError::TokenAccountIsNotRegistered);
            }
        };
        if yocto_token_balance < yocto_token_amount {
            return Err(BaseError::InsufficientTokenAccountBalance);
        }
        yocto_token_balance -= yocto_token_amount;

        self.management_fund.available_for_staking_balance -= yocto_near_amount;

        if yocto_token_balance > 0 {
            self.fungible_token.token_account_registry.insert(&account_id, &yocto_token_balance);
        } else {
            if let None = self.fungible_token.token_account_registry.remove(&account_id) {
                return Err(BaseError::Logic);
            }
            self.fungible_token.token_accounts_quantity -= 1;

            yocto_near_amount += Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_token_account)?;
        }

        self.fungible_token.total_supply -= yocto_token_amount;

        Ok(
            Promise::new(account_id)
                .transfer(yocto_near_amount)
        )
    }

    fn internal_delayed_withdraw(&mut self, yocto_token_amount: u128) -> Result<Promise, BaseError> {
        self.assert_epoch_is_synchronized()?;

        let account_id = env::predecessor_account_id();

        if yocto_token_amount == 0 {
            return Err(BaseError::InsufficientTokenDeposit);
        }

        let mut yocto_near_amount = self.convert_yocto_token_amount_to_yocto_near_amount(yocto_token_amount)?;
        if yocto_near_amount == 0 {
            return Err(BaseError::InsufficientTokenDeposit);
        }
        if yocto_near_amount > self.management_fund.available_for_staking_balance {
            return Err(BaseError::InsufficientAvailableForStakingBalance);
        }

        let account_yocto_token_amount = match self.fungible_token.token_account_registry.get(&account_id) {
            Some(yocto_token_balance_) => yocto_token_balance_,
            None => {
                return Err(BaseError::TokenAccountIsNotRegistered);
            }
        };

        if yocto_token_amount > account_yocto_token_amount {
            return Err(BaseError::InsufficientTokenAccountBalance);
        }

        todo!();
    }

    fn internal_add_validator(
        &mut self,
        validator_account_id: AccountId,
        validator_staking_contract_version: ValidatorStakingContractVersion,
        delayed_unstake_validator_group: DelayedUnstakeValidatorGroup
    ) -> Result<(), BaseError> {   // TODO можно ли проверить, что адрес валиден, и валидатор в вайт-листе?
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        let storage_staking_price_per_additional_validator_account = Self::calculate_storage_staking_price(self.validating_node.storage_usage_per_validator_account)?;
        if env::attached_deposit() < storage_staking_price_per_additional_validator_account {
            return Err(BaseError::InsufficientNearDepositForStorageStaking);
        }

        if let Some(maximium_quantity) = self.validating_node.validator_accounts_maximum_quantity {
            if self.validating_node.validator_accounts_quantity == maximium_quantity {
                return Err(BaseError::ValidatorAccountsMaximumQuantityExceeding);
            }
        }

        if let Some(_) = self.validating_node.validator_account_registry.insert(
            &validator_account_id, &ValidatorInfo::new(validator_staking_contract_version, delayed_unstake_validator_group)
        ) {
            return Err(BaseError::ValidatorAccountIsAlreadyRegistered);
        }
        self.validating_node.validator_accounts_quantity += 1;
        self.validating_node.quantity_of_validators_accounts_updated_in_current_epoch += 1;     // TODO вот это точно ли нужно

        let yocto_near_amount = env::attached_deposit() - storage_staking_price_per_additional_validator_account;
        if yocto_near_amount > 0 {
            Promise::new(env::predecessor_account_id())
                .transfer(yocto_near_amount);   // TODO Нужен ли коллбек?
        }

        Ok(())
    }

    fn internal_remove_validator(&mut self, validator_account_id: AccountId) -> Result<Promise, BaseError> {
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        match self.validating_node.validator_account_registry.remove(&validator_account_id) {
            Some(validator_info) => {
                if validator_info.staked_balance > 0 {
                    return Err(BaseError::RemovingValidatorWithExistingBalance);
                }
            }
            None => {
                return Err(BaseError::ValidatorAccountIsNotRegistered);
            }
        }

        self.validating_node.validator_accounts_quantity -= 1;
        self.validating_node.quantity_of_validators_accounts_updated_in_current_epoch -= 1;    // TODO  вот это точно ли нужно относительно internal_add_validator

        Ok(
            Promise::new(env::predecessor_account_id())
                .transfer(Self::calculate_storage_staking_price(self.validating_node.storage_usage_per_validator_account)?)
        )
    }

    fn internal_increase_validator_stake(
        &mut self, validator_account_id: AccountId, yocto_near_amount: Balance
    ) -> Result<Promise, BaseError> {      // TODO Сюда нужно зафиксировать максимальное число Газа. Возможно ли?
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        if self.management_fund.available_for_staking_balance== 0
            || !(1..=self.management_fund.available_for_staking_balance).contains(&yocto_near_amount) {
            return Err(BaseError::InsufficientAvailableForStakingBalance);
        }

        // let deposit_and_stake_gas = Gas(ONE_TERA * Self::DEPOSIT_AND_STAKE_TGAS);           // TODO проверка, сколько газа прикрепили

        match self.validating_node.validator_account_registry.get(&validator_account_id) {
            Some(validator_info) => {
                match validator_info.staking_contract_version {
                    ValidatorStakingContractVersion::Classic => {
                        return Ok(
                            ext_staking_pool::ext(validator_account_id.clone())
                                .with_attached_deposit(yocto_near_amount)
                                // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                                .deposit_and_stake()
                                .then(
                                    Self::ext(env::current_account_id())           // TODO TODO TODO TODO точно ли этот аккаунт
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

    fn internal_update_validator_info(      // TODO TODO TODO Что делать, если в новой эпохе часть обновилась, и уже еще раз наступила новая эпоха, и теперь то, что осталось, обновились. То есть, рассинхронизация состояния.
        &mut self, validator_account_id: AccountId
    ) -> Result<Promise, BaseError> {      // TODO Сюда нужно зафиксировать максимальное число Газа. Возможно ли?
        self.assert_epoch_is_desynchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        match self.validating_node.validator_account_registry.get(&validator_account_id) {   // TODO // TODO ЧТо будет, если валидатор перестал работать, что придет с контракта. Не прервется ли из-за этго цепочка выполнения апдейтов
            Some(validator_info) => {
                let current_epoch_height = env::epoch_height();

                if validator_info.last_update_info_epoch_height < current_epoch_height {
                    match validator_info.staking_contract_version {
                        ValidatorStakingContractVersion::Classic => {
                            return Ok(
                                ext_staking_pool::ext(validator_account_id.clone())
                                    // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                                    .get_account_total_balance(env::current_account_id())
                                    .then(
                                        Self::ext(env::current_account_id())           // TODO TODO TODO TODO  смотреть, точно ли этот адрес
                                            .update_validator_info_callback(&validator_account_id, current_epoch_height)
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

    fn internal_update(&mut self) -> Result<(), BaseError>{
        self.assert_epoch_is_desynchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        if self.validating_node.quantity_of_validators_accounts_updated_in_current_epoch != self.validating_node.validator_accounts_quantity {
            return Err(BaseError::SomeValidatorInfoDoesNotUpdated);
        }

        let previous_epoch_rewards_from_validators_yocto_token_amount = self.convert_yocto_near_amount_to_yocto_token_amount(
            self.previous_epoch_rewards_from_validators_yocto_near_amount
        )?;

        self.management_fund.staked_balance += self.previous_epoch_rewards_from_validators_yocto_near_amount;
        self.management_fund.is_distributed_on_validators_in_current_epoch = false;
        self.validating_node.current_delayed_unstake_validator_group.set_next();
        self.validating_node.quantity_of_validators_accounts_updated_in_current_epoch = 0;
        self.current_epoch_height = env::epoch_height();
        self.increase_total_rewards_from_validators_yocto_near_amount(self.previous_epoch_rewards_from_validators_yocto_near_amount)?;
        self.previous_epoch_rewards_from_validators_yocto_near_amount = 0;

        if let Some(ref rewards_fee) = self.fee_registry.rewards_fee {
            let rewards_fee_yocto_token_amount = rewards_fee.multiply(previous_epoch_rewards_from_validators_yocto_token_amount);
            if rewards_fee_yocto_token_amount != 0 {
                match self.fungible_token.token_account_registry.get(&self.rewards_receiver_account_id) {
                    Some(mut yocto_token_balance) => {
                        yocto_token_balance += rewards_fee_yocto_token_amount;

                        self.fungible_token.total_supply += rewards_fee_yocto_token_amount;
                        self.fungible_token.token_account_registry.insert(&self.rewards_receiver_account_id, &yocto_token_balance);
                    }
                    None => {
                        return Err(BaseError::Logic);
                    }
                }
            }

            if let Some(ref everstake_rewards_fee) = self.fee_registry.everstake_rewards_fee {
                let everstake_rewards_fee_yocto_token_amount = everstake_rewards_fee.multiply(rewards_fee_yocto_token_amount);
                if everstake_rewards_fee_yocto_token_amount != 0 {
                    match self.fungible_token.token_account_registry.get(&self.everstake_rewards_receiver_account_id) {
                        Some(mut yocto_token_balance) => {
                            yocto_token_balance += everstake_rewards_fee_yocto_token_amount;

                            self.fungible_token.total_supply += everstake_rewards_fee_yocto_token_amount;
                            self.fungible_token.token_account_registry.insert(&self.everstake_rewards_receiver_account_id, &yocto_token_balance);
                        }
                        None => {
                            return Err(BaseError::Logic);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn internal_change_manager(&mut self, manager_id: AccountId) -> Result<(), BaseError> {
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management()?;

        self.manager_id = manager_id;

        Ok(())
    }

    fn internal_change_rewards_fee(&mut self, rewards_fee: Option<Fee>) -> Result<(), BaseError> {
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        if let Some(ref rewards_fee_) = rewards_fee {
            rewards_fee_.assert_valid()?;
        }

        self.fee_registry.rewards_fee = rewards_fee;

        Ok(())
    }

    fn internal_change_everstake_rewards_fee(&mut self, everstake_rewards_fee: Option<Fee>) -> Result<(), BaseError> {
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        if let Some(ref everstake_rewards_fee_) = everstake_rewards_fee {
            everstake_rewards_fee_.assert_valid()?;
        }

        self.fee_registry.everstake_rewards_fee = everstake_rewards_fee;

        Ok(())
    }

    fn internal_is_account_registered(&self, account_id: AccountId) -> bool {
        self.fungible_token.token_account_registry.contains_key(&account_id)
    }

    fn internal_get_total_token_supply(&self) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(self.fungible_token.total_supply)
    }

    fn internal_get_stakers_quantity(&self) -> u64 {
        self.fungible_token.token_accounts_quantity
    }

    fn internal_get_storage_staking_price_per_additional_token_account(&self) -> Result<Balance, BaseError> {
        Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_token_account)
    }

    fn internal_get_yocto_token_amount_from_yocto_near_amount(&self, yocto_near_amount: Balance) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        self.convert_yocto_near_amount_to_yocto_token_amount(yocto_near_amount)
    }

    fn internal_get_yocto_near_amount_from_yocto_token_amount(&self, yocto_token_amount: Balance) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        self.convert_yocto_token_amount_to_yocto_near_amount(yocto_token_amount)
    }

    fn internal_get_token_account_balance(&self, account_id: AccountId) -> Result<Balance, BaseError> {
        match self.fungible_token.token_account_registry.get(&account_id) {
            Some(yocto_token_balance_) => Ok(yocto_token_balance_),
            None => {
                return Err(BaseError::TokenAccountIsNotRegistered);
            }
        }
    }

    fn internal_get_available_for_staking_balance(&self) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(self.management_fund.available_for_staking_balance)
    }

    fn internal_get_staked_balance(&self) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(self.management_fund.staked_balance)
    }

    fn internal_get_management_fund_amount(&self) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(self.management_fund.get_management_fund_amount())
    }

    fn internal_get_fee_registry(&self) -> Result<FeeRegistry, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(self.fee_registry.clone())
    }

    pub fn internal_get_current_epoch_height(&self) -> (EpochHeight, EpochHeight) {
        (self.current_epoch_height, env::epoch_height())
    }

    fn internal_get_validator_info_dto(&self) -> Vec<ValidatorInfoDto> {
        let mut validator_info_dto_registry: Vec<ValidatorInfoDto> = vec![];

        for (account_id, validator_info) in self.validating_node.validator_account_registry.into_iter() {
            let ValidatorInfo {
                delayed_unstake_validator_group: _,
                staking_contract_version: _,
                staked_balance,
                last_update_info_epoch_height,
                last_stake_increasing_epoch_height
            } = validator_info;

            validator_info_dto_registry.push(
                ValidatorInfoDto {
                    account_id,
                    staked_balance: staked_balance.into(),
                    last_update_info_epoch_height,
                    last_stake_increasing_epoch_height
                }
            );
        }

        validator_info_dto_registry
    }

    fn internal_get_aggregated_information(&self) -> Result<AggregatedInformation, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(
            AggregatedInformation {
                available_for_staking_balance: self.management_fund.available_for_staking_balance.into(),
                staked_balance: self.management_fund.staked_balance.into(),
                token_total_supply: self.fungible_token.total_supply.into(),
                token_accounts_quantity: self.fungible_token.token_accounts_quantity,
                total_rewards_from_validators_yocto_near_amount: self.total_rewards_from_validators_yocto_near_amount.into(),
                rewards_fee: self.fee_registry.rewards_fee.clone()
            }
        )
    }

    fn internal_confirm_stake_distribution(&mut self) -> Result<(), BaseError> {
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        self.management_fund.is_distributed_on_validators_in_current_epoch = true;

        Ok(())
    }

    fn convert_yocto_near_amount_to_yocto_token_amount(&self, yocto_near_amount: Balance) -> Result<Balance, BaseError> {
        if self.management_fund.get_management_fund_amount() == 0 {
            return Ok(yocto_near_amount);
        }

        Ok(                  // TODO Проверить Округление
            (
                U256::from(yocto_near_amount)
                * U256::from(self.fungible_token.total_supply)
                / U256::from(self.management_fund.get_management_fund_amount())
            ).as_u128()
        )
    }

    fn convert_yocto_token_amount_to_yocto_near_amount(&self, yocto_token_amount: Balance) -> Result<Balance, BaseError> {
        if self.fungible_token.total_supply == 0 {
            return Ok(yocto_token_amount);
        }

        Ok(         // TODO Проверить Округление
            (
                U256::from(yocto_token_amount)
                * U256::from(self.management_fund.get_management_fund_amount())
                / U256::from(self.fungible_token.total_supply)
            ).as_u128()
        )
    }

    fn assert_authorized_management_only_by_manager(&self) -> Result<(), BaseError> {
        if self.manager_id != env::predecessor_account_id() {
            return Err(BaseError::UnauthorizedManagementOnlyByManager);
        }

        Ok(())
    }

    fn assert_authorized_management(&self) -> Result<(), BaseError> {
        let predecessor_account_id = env::predecessor_account_id();

        if self.owner_id == predecessor_account_id || self.manager_id == predecessor_account_id {
            return Ok(());
        }

        Err(BaseError::UnauthorizedManagement)
    }

    fn assert_epoch_is_synchronized(&self) -> Result<(), BaseError> {
        if self.current_epoch_height != env::epoch_height() {
            return Err(BaseError::DesynchronizedEpoch);
        }

        Ok(())
    }

    fn assert_epoch_is_desynchronized(&self) -> Result<(), BaseError> {
        if self.current_epoch_height == env::epoch_height() {
            return Err(BaseError::SynchronizedEpoch);
        }

        Ok(())
    }

    fn increase_total_rewards_from_validators_yocto_near_amount(&mut self, yocto_near_amount: Balance) -> Result<(), BaseError> {
        self.total_rewards_from_validators_yocto_near_amount = match self.total_rewards_from_validators_yocto_near_amount
            .checked_add(yocto_near_amount) {
            Some(total_rewards_from_validators_yocto_near_amount_) => {
                total_rewards_from_validators_yocto_near_amount_
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        };

        Ok(())
    }

    fn calculate_storage_staking_price(quantity_of_bytes: StorageUsage) -> Result<Balance, BaseError> {
        match Balance::from(quantity_of_bytes).checked_mul(env::storage_byte_cost()) {
            Some(value) => {
                Ok(value)
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        }
    }
}

#[near_bindgen]
impl StakePool {
    #[init]
    pub fn new(
        manager_id: Option<AccountId>,
        rewards_receiver_account_id: AccountId,
        everstake_rewards_receiver_account_id: AccountId,
        rewards_fee: Option<Fee>,
        everstake_rewards_fee: Option<Fee>,
        validators_maximum_quantity: Option<u64>
    ) -> Self {      // TODO Сюда заходит дипозит. Как его отсечь, то есть, взять ту часть, к
        match Self::internal_new(
            manager_id, rewards_receiver_account_id, everstake_rewards_receiver_account_id, rewards_fee, everstake_rewards_fee, validators_maximum_quantity
        ) {
            Ok(stake_pool) => {
                stake_pool
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    /// Stake process.
    #[payable]
    pub fn deposit(&mut self) {
        if let Err(error) = self.internal_deposit() {               // TODO TODO ЕСЛИ при distribute_available_for_staking_balance уже распределено, то здесь сразу распределеям на случайный валидатор из текущей группы
            env::panic_str(format!("{}", error).as_str());
        }
    }

    /// Instant unstake process.
    pub fn instant_withdraw(&mut self, yocto_token_amount: U128) -> Promise {
        match self.internal_instant_withdraw(yocto_token_amount.into()) {
            Ok(promise) => {
                promise
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    /// Delayed unstake process.
    pub fn delayed_withdraw(&mut self, yocto_token_amount: U128) -> Promise {
        match self.internal_delayed_withdraw(yocto_token_amount.into()) {
            Ok(promise) => {
                promise
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    #[payable]
    pub fn add_validator(
        &mut self,
        validator_account_id: AccountId,
        validator_staking_contract_version: ValidatorStakingContractVersion,
        delayed_unstake_validator_group: DelayedUnstakeValidatorGroup
    ) {
        if let Err(error) = self.internal_add_validator(
            validator_account_id, validator_staking_contract_version, delayed_unstake_validator_group
        ) {
            env::panic_str(format!("{}", error).as_str());
        }
    }

    pub fn remove_validator(&mut self, validator_account_id: AccountId) -> Promise {
        match self.internal_remove_validator(validator_account_id) {
            Ok(promise) => {
                promise
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn increase_validator_stake(
        &mut self, validator_account_id: AccountId, yocto_near_amount: Balance
    ) -> Promise {
        match self.internal_increase_validator_stake(validator_account_id, yocto_near_amount) {
            Ok(promise) => {
                promise
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn update_validator_info(
        &mut self, validator_account_id: AccountId
    ) -> Promise {
        match self.internal_update_validator_info(validator_account_id) {
            Ok(promise) => {
                promise
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn update(&mut self) {
        if let Err(error) = self.internal_update() {
            env::panic_str(format!("{}", error).as_str());
        }
    }

    pub fn change_manager(&mut self, manager_id: AccountId) {
        if let Err(error) = self.internal_change_manager(manager_id) {
            env::panic_str(format!("{}", error).as_str());
        }
    }

    pub fn change_rewards_fee(&mut self, rewards_fee: Option<Fee>) {
        if let Err(error) = self.internal_change_rewards_fee(rewards_fee) {
            env::panic_str(format!("{}", error).as_str());
        }
    }

    pub fn change_everstake_rewards_fee(&mut self, everstake_rewards_fee: Option<Fee>) {
        if let Err(error) = self.internal_change_everstake_rewards_fee(everstake_rewards_fee) {
            env::panic_str(format!("{}", error).as_str());
        }
    }

    pub fn is_account_registered(&self, account_id: AccountId) -> bool {
        self.internal_is_account_registered(account_id)
    }

    pub fn confirm_stake_distribution(&mut self) {
        if let Err(error) = self.internal_confirm_stake_distribution() {
            env::panic_str(format!("{}", error).as_str());
        }
    }

    pub fn get_total_token_supply(&self) -> U128 {
        match self.internal_get_total_token_supply() {
            Ok(total_token_supply) => {
                total_token_supply.into()
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn get_stakers_quantity(&self) -> u64 {
        self.internal_get_stakers_quantity()
    }

    pub fn get_storage_staking_price_per_additional_token_account(&self) -> U128 {
        match self.internal_get_storage_staking_price_per_additional_token_account() {
            Ok(storage_staking_price) => {
                storage_staking_price.into()
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn get_yocto_token_amount_from_yocto_near_amount(&self, yocto_near_amount: U128) -> U128 {
        match self.internal_get_yocto_token_amount_from_yocto_near_amount(yocto_near_amount.into()) {
            Ok(yocto_token_amount) => {
                yocto_token_amount.into()
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn get_yocto_near_amount_from_yocto_token_amount(&self, yocto_token_amount: U128) -> U128 {
        match self.internal_get_yocto_near_amount_from_yocto_token_amount(yocto_token_amount.into()) {
            Ok(yocto_near_amount) => {
                yocto_near_amount.into()
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn get_token_account_balance(&self, account_id: AccountId) -> U128 {
        match self.internal_get_token_account_balance(account_id) {
            Ok(token_account_balance) => {
                token_account_balance.into()
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn get_available_for_staking_balance(&self) -> U128 {
        match self.internal_get_available_for_staking_balance() {
            Ok(available_for_staking_balance) => {
                available_for_staking_balance.into()
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn get_staked_balance(&self) -> U128 {
        match self.internal_get_staked_balance() {
            Ok(staked_balance) => {
                staked_balance.into()
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn get_management_fund_amount(&self) -> U128 {
        match self.internal_get_management_fund_amount() {
            Ok(management_fund_amount) => {
                management_fund_amount.into()
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn get_fee_registry(&self) -> FeeRegistry {
        match self.internal_get_fee_registry() {
            Ok(fee_registry) => {
                fee_registry
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn get_current_epoch_height(&self) -> (EpochHeight, EpochHeight) {
        self.internal_get_current_epoch_height()
    }

    pub fn is_stake_distributed(&self) -> bool {
        self.management_fund.is_distributed_on_validators_in_current_epoch
    }

    pub fn get_validator_info_dto(&self) -> Vec<ValidatorInfoDto> { // TODO есть Info , есть Information (проблема в имени)
        self.internal_get_validator_info_dto()
    }

    pub fn get_aggregated_information(&self) -> AggregatedInformation { // TODO есть Info , есть Information (проблема в имени)
        match self.internal_get_aggregated_information() {
            Ok(aggregated_information) => {
                aggregated_information
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }
}

#[near_bindgen]
impl StakePool {
    #[private]
    pub fn increase_validator_stake_callback(
        &mut self,
        validator_account_id: &AccountId,
        staked_yocto_near_amount: Balance,
        current_epoch_height: EpochHeight
    ) -> bool {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");        // TODO Фраза повторяется. Нужно ли выновсить в константу?
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                self.management_fund.available_for_staking_balance -= staked_yocto_near_amount;
                self.management_fund.staked_balance += staked_yocto_near_amount;

                let mut validator_info = self.validating_node.validator_account_registry.get(validator_account_id).unwrap();  // TODO unwrap     МОЖНО ПереДАВАТЬ в КОЛЛБЭК этот объектОБЪЕКТ Сразу
                validator_info.staked_balance += staked_yocto_near_amount;
                validator_info.last_stake_increasing_epoch_height = Some(current_epoch_height);
                self.validating_node.validator_account_registry.insert(validator_account_id, &validator_info);

                true
            }
            _ => {
                false
            }
        }
    }

    // TODO комментарий написать. Возвращаем и сохраняем епохи в разном состоянии по-разному, чтобы запомнить, что в какой эпохе инициировано по фактту, а в какую выполнен коллбек
    #[private]
    pub fn update_validator_info_callback(
        &mut self,
        validator_account_id: &AccountId,
        current_epoch_height: EpochHeight
    ) -> (bool, EpochHeight) {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(data) => {
                let new_staked_balance: u128 = near_sdk::serde_json::from_slice::<U128>(data.as_slice()).unwrap().into();          // TODO Что делать с Анврепом

                let mut validator_info = self.validating_node.validator_account_registry.get(validator_account_id).unwrap();  // TODO unwrap

                let current_staked_balance = validator_info.staked_balance;

                let staking_rewards_yocto_near_amount = if new_staked_balance >= current_staked_balance {
                    new_staked_balance - current_staked_balance
                } else {
                    env::panic_str("Contract logic error.");        // TODO  как обоработать. Может, возвращать структуры ?
                };

                validator_info.last_update_info_epoch_height = current_epoch_height;
                validator_info.staked_balance = new_staked_balance;

                self.validating_node.validator_account_registry.insert(validator_account_id, &validator_info);
                self.validating_node.quantity_of_validators_accounts_updated_in_current_epoch += 1;

                self.management_fund.staked_balance += staking_rewards_yocto_near_amount;
                self.previous_epoch_rewards_from_validators_yocto_near_amount += staking_rewards_yocto_near_amount;

                (true, env::epoch_height())
            }
            _ => {
                (false, env::epoch_height())
            }
        }
    }
}

// TODO  Добавить к системным Промисам Коллбэк (логирование или подобное)

// TODO проставить проверку по типу amount>0.

// TODO Понять работу с аккаунтами. КОму пренадлжат, кто может мзенять состояние, и подобные вещи

// #[ext_contract(ext_voting)]           что это такое???????????????????????????????????????????????
// pub trait VoteContract {
//     /// Method for validators to vote or withdraw the vote.
//     /// Votes for if `is_vote` is true, or withdraws the vote if `is_vote` is false.
//     fn vote(&mut self, is_vote: bool);
// }


//#[global_allocator]
// static ALLOC: near_sdk::wee_alloc::WeeAlloc = near_sdk::wee_alloc::WeeAlloc::INIT;            Нужно ли вот это ??????????????????


// Returning Promise: This allows NEAR Explorer, near-cli, near-api-js, and other tooling to correctly determine if a whole chain of transactions
// is successful. If your function does not return Promise, tools like near-cli will return immediately after your function call.
// And then even if the transfer fails, your function call will be considered successful. You can see a before & after example of this behavior here.





// TODO IMPORTANT!!!!!!!!!!!!!!!!!!!!!!!
// WhiteList
// Managment Secuirity
// Multisig key
// Container deploying
// Mint authority checking


// TODO пройтись по именам полей. Валидатор - это стекинг, а не сам валидаьор, например

// TODO CLIPPY
// TODO ПАоменять стиль матчинга

// TODO NOTE: stake pool Guarantees are based on the no-slashing condition. Once slashing is introduced, the contract will no longer
// provide some guarantees. Read more about slashing in [Nightshade paper](https://near.ai/nightshade).