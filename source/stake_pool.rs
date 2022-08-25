use core::convert::Into;
use near_sdk::{env, near_bindgen, PanicOnDefault, AccountId, Balance, EpochHeight, Promise, PromiseResult};
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
use super::validator_staking_contract_version::ValidatorStakingContractVersion;
use uint::construct_uint;

construct_uint! {
    pub struct U256(4);
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]     // TODO проверить все типы данных. LazyOption, например, добавить. !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!1
pub struct StakePool {
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

        Ok(
            Self {
                owner_id: env::predecessor_account_id(),
                manager_id: manager_id_,
                rewards_receiver_account_id,
                everstake_rewards_receiver_account_id,
                fee_registry: FeeRegistry::new(rewards_fee, everstake_rewards_fee),
                fungible_token: FungibleToken::new(env::predecessor_account_id())?,
                management_fund: ManagementFund::new(),
                validating_node: ValidatingNode::new(validators_maximum_quantity)?,
                current_epoch_height: env::epoch_height(),
                previous_epoch_rewards_from_validators_yocto_near_amount: 0,
                total_rewards_from_validators_yocto_near_amount: 0
            }
        )
    }

    fn internal_deposit(&mut self) -> Result<(), BaseError> {
        self.assert_epoch_is_synchronized()?;

        let account_id = env::predecessor_account_id();

        let mut net_convertible_yocto_near_amount = env::attached_deposit();
        if !self.fungible_token.is_token_account_registered(&account_id) {
            let storage_staking_price_per_additional_token_account = self.fungible_token.get_storage_staking_price_per_additional_token_account()?;
            if net_convertible_yocto_near_amount < storage_staking_price_per_additional_token_account {
                return Err(BaseError::InsufficientNearDepositForStorageStaking);
            }
            net_convertible_yocto_near_amount = net_convertible_yocto_near_amount - storage_staking_price_per_additional_token_account;

            self.fungible_token.register_token_account(&account_id)?;
        }
        if net_convertible_yocto_near_amount == 0 {
            return Err(BaseError::InsufficientNearDeposit);
        }

        let yocto_token_amount = self.convert_yocto_near_amount_to_yocto_token_amount(net_convertible_yocto_near_amount)?;
        if yocto_token_amount == 0 {
            return Err(BaseError::InsufficientNearDeposit);
        }

        self.fungible_token.increase_token_account_balance(&account_id, yocto_token_amount)?;
        self.management_fund.increase_available_for_staking_balance(net_convertible_yocto_near_amount)?;

        Ok(())
    }

    fn internal_instant_withdraw(&mut self, yocto_token_amount: u128) -> Result<Promise, BaseError> {
        self.assert_epoch_is_synchronized()?;

        let account_id = env::predecessor_account_id();

        if yocto_token_amount == 0 {
            return Err(BaseError::InsufficientTokenDeposit);
        }

        let mut yocto_near_amount = self.convert_yocto_token_amount_to_yocto_near_amount(yocto_token_amount)?;
        if yocto_near_amount == 0 {
            return Err(BaseError::InsufficientTokenDeposit);
        }

        self.fungible_token.decrease_token_account_balance(&account_id, yocto_token_amount)?;
        self.management_fund.decrease_available_for_staking_balance(yocto_near_amount)?;
        if self.fungible_token.can_unregister_token_account(&account_id)? {
            self.fungible_token.unregister_token_account(&account_id)?;

            yocto_near_amount = yocto_near_amount + self.fungible_token.get_storage_staking_price_per_additional_token_account()?;
        }

        Ok(
            Promise::new(account_id)
                .transfer(yocto_near_amount)
        )
    }

    fn internal_add_validator(
        &mut self,
        validator_account_id: AccountId,
        validator_staking_contract_version: ValidatorStakingContractVersion,
        delayed_unstake_validator_group: DelayedUnstakeValidatorGroup
    ) -> Result<(), BaseError> {   // TODO можно ли проверить, что адрес валиден, и валидатор в вайт-листе?
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        let storage_staking_price_per_additional_validator_account = self.validating_node.get_storage_staking_price_per_additional_validator_account()?;
        if env::attached_deposit() < storage_staking_price_per_additional_validator_account {
            return Err(BaseError::InsufficientNearDepositForStorageStaking);
        }
        self.validating_node.register_validator_account(&validator_account_id, validator_staking_contract_version, delayed_unstake_validator_group)?;

        let yocto_near_amount = env::attached_deposit() - storage_staking_price_per_additional_validator_account;
        if yocto_near_amount > 0 {
            Promise::new(env::predecessor_account_id())
                .transfer(yocto_near_amount);   // TODO написать в коллбеке ретурн отсюда
        }

        Ok(())
    }

    fn internal_remove_validator(&mut self, validator_account_id: AccountId) -> Result<Promise, BaseError> {
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        self.validating_node.unregister_validator_account(&validator_account_id)?;

        Ok(
            Promise::new(env::predecessor_account_id())
                .transfer(self.validating_node.get_storage_staking_price_per_additional_validator_account()?)
        )
    }

    fn internal_increase_validator_stake(
        &mut self, validator_account_id: AccountId, yocto_near_amount: Balance
    ) -> Result<Promise, BaseError> {      // TODO Сюда нужно зафиксировать максимальное число Газа. Возможно ли?
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        let available_for_staking_balance = self.management_fund.get_available_for_staking_balance();
        if available_for_staking_balance == 0 || !(1..=available_for_staking_balance).contains(&yocto_near_amount) {
            return Err(BaseError::InsufficientAvailableForStakingBalance);
        }

        self.validating_node.increase_validator_stake(&validator_account_id, yocto_near_amount)
    }

    fn internal_update_validator_info(      // TODO TODO TODO Что делать, если в новой эпохе часть обновилась, и уже еще раз наступила новая эпоха, и теперь то, что осталось, обновились. То есть, рассинхронизация состояния.
        &mut self, validator_account_id: AccountId
    ) -> Result<Promise, BaseError> {      // TODO Сюда нужно зафиксировать максимальное число Газа. Возможно ли?
        self.assert_epoch_is_desynchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        self.validating_node.update_validator_info(&validator_account_id)
    }

    fn internal_update(&mut self) -> Result<(), BaseError>{
        self.assert_epoch_is_desynchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        if !self.validating_node.is_all_validators_updated_in_current_epoch() {
            return Err(BaseError::SomeValidatorInfoDoesNotUpdated);
        }

        let previous_epoch_rewards_from_validators_yocto_token_amount = self.convert_yocto_near_amount_to_yocto_token_amount(
            self.previous_epoch_rewards_from_validators_yocto_near_amount
        )?;

        self.management_fund.increase_staked_balance(self.previous_epoch_rewards_from_validators_yocto_near_amount)?;
        self.validating_node.update();
        self.current_epoch_height = env::epoch_height();
        self.increase_total_rewards_from_validators_yocto_near_amount(self.previous_epoch_rewards_from_validators_yocto_near_amount)?;
        self.previous_epoch_rewards_from_validators_yocto_near_amount = 0;

        if let Some(rewards_fee) = self.fee_registry.get_rewards_fee() {
            let rewards_fee_yocto_token_amount = rewards_fee.multiply(previous_epoch_rewards_from_validators_yocto_token_amount);
            if rewards_fee_yocto_token_amount != 0 {
                if !self.fungible_token.is_token_account_registered(&self.rewards_receiver_account_id) {
                    self.fungible_token.register_token_account(&self.rewards_receiver_account_id)?;
                }

                self.fungible_token.increase_token_account_balance(&self.rewards_receiver_account_id, rewards_fee_yocto_token_amount)?;
            }

            if let Some(everstake_rewards_fee) = self.fee_registry.get_everstake_rewards_fee() {
                let everstake_rewards_fee_yocto_token_amount = everstake_rewards_fee.multiply(rewards_fee_yocto_token_amount);
                if everstake_rewards_fee_yocto_token_amount != 0 {
                    if !self.fungible_token.is_token_account_registered(&self.everstake_rewards_receiver_account_id) {
                        self.fungible_token.register_token_account(&self.everstake_rewards_receiver_account_id)?;
                    }

                    self.fungible_token.increase_token_account_balance(&self.everstake_rewards_receiver_account_id, everstake_rewards_fee_yocto_token_amount)?;
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

        self.fee_registry.change_rewards_fee(rewards_fee);

        Ok(())
    }

    fn internal_change_everstake_rewards_fee(&mut self, everstake_rewards_fee: Option<Fee>) -> Result<(), BaseError> {
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        if let Some(ref everstake_rewards_fee_) = everstake_rewards_fee {
            everstake_rewards_fee_.assert_valid()?;
        }

        self.fee_registry.change_everstake_rewards_fee(everstake_rewards_fee);

        Ok(())
    }

    fn internal_is_account_registered(&self, account_id: AccountId) -> bool {
        self.fungible_token.is_token_account_registered(&account_id)
    }

    fn internal_get_total_token_supply(&self) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(self.fungible_token.get_total_token_supply())
    }

    fn internal_get_stakers_quantity(&self) -> u64 {
        self.fungible_token.get_token_accounts_quantity()
    }

    fn internal_get_storage_staking_price_per_additional_token_account(&self) -> Result<Balance, BaseError> {
        self.fungible_token.get_storage_staking_price_per_additional_token_account()
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
        self.fungible_token.get_token_account_balance(&account_id)
    }

    fn internal_get_available_for_staking_balance(&self) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(self.management_fund.get_available_for_staking_balance())
    }

    fn internal_get_staked_balance(&self) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(self.management_fund.get_staked_balance())
    }

    fn internal_get_management_fund_amount(&self) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(self.management_fund.get_management_fund_amount()?)
    }

    fn internal_get_fee_registry(&self) -> Result<FeeRegistry, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(self.fee_registry.clone())
    }

    pub fn internal_get_current_epoch_height(&self) -> EpochHeight {
        self.current_epoch_height
    }

    fn internal_get_validator_info_dto(&self) -> Vec<ValidatorInfoDto> {
        self.validating_node.get_validator_info_dto()
    }

    fn internal_get_aggregated_information(&self) -> Result<AggregatedInformation, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(
            AggregatedInformation::new(
                self.management_fund.get_available_for_staking_balance().into(),
                self.management_fund.get_staked_balance().into(),
                self.fungible_token.get_total_token_supply().into(),
                self.fungible_token.get_token_accounts_quantity(),
                self.total_rewards_from_validators_yocto_near_amount.into(),
                self.fee_registry.get_rewards_fee().clone()
            )
        )
    }

    fn convert_yocto_near_amount_to_yocto_token_amount(&self, yocto_near_amount: Balance) -> Result<Balance, BaseError> {
        if self.management_fund.get_management_fund_amount()? == 0 {
            return Ok(yocto_near_amount);
        }

        Ok(                  // TODO Проверить Округление
            (
                U256::from(yocto_near_amount)
                * U256::from(self.fungible_token.get_total_token_supply())
                / U256::from(self.management_fund.get_management_fund_amount()?)
            ).as_u128()
        )
    }

    fn convert_yocto_token_amount_to_yocto_near_amount(&self, yocto_token_amount: Balance) -> Result<Balance, BaseError> {
        if self.fungible_token.get_total_token_supply() == 0 {
            return Ok(yocto_token_amount);
        }

        Ok(         // TODO Проверить Округление
            (
                U256::from(yocto_token_amount)
                * U256::from(self.management_fund.get_management_fund_amount()?)
                / U256::from(self.fungible_token.get_total_token_supply())
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

    pub fn increase_previous_epoch_rewards_from_validators_yocto_near_amount(&mut self, yocto_near_amount: Balance) -> Result<(), BaseError> {
        self.previous_epoch_rewards_from_validators_yocto_near_amount = match self.previous_epoch_rewards_from_validators_yocto_near_amount
            .checked_add(yocto_near_amount) {
            Some(previous_epoch_rewards_from_validators_yocto_near_amount_) => {
                previous_epoch_rewards_from_validators_yocto_near_amount_
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        };

        Ok(())
    }

    pub fn get_management_fund(&mut self) -> &mut ManagementFund {
        &mut self.management_fund
    }

    pub fn get_validating_node(&mut self) -> &mut ValidatingNode {
        &mut self.validating_node
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
        // match self.internal_delayed_withdraw(yocto_token_amount.into()) {
        //     Ok(promise) => {
        //         promise
        //     }
        //     Err(error) => {
        //         env::panic_str(format!("{}", error).as_str());
        //     }
        // }

        todo!();
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

    pub fn get_current_epoch_height(&self) -> EpochHeight {
        self.internal_get_current_epoch_height()
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