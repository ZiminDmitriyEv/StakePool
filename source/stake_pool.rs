use core::convert::Into;
use near_sdk::{env, near_bindgen, PanicOnDefault, AccountId, Balance, EpochHeight, Promise, PromiseResult, StorageUsage};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::json_types::U128;
use super::aggregated_information_dto::AggregatedInformationDto;
use super::base_error::BaseError;
use super::delayed_withdrawal_info_dto::DelayedWithdrawalInfoDto;
use super::delayed_withdrawal_info::DelayedWithdrawalInfo;
use super::EPOCH_QUANTITY_TO_DELAYED_WITHDRAWAL;
use super::fee_registry::FeeRegistry;
use super::fee::Fee;
use super::fungible_token_registry_dto::FungibleTokenRegistryDto;
use super::fungible_token_registry::FungibleTokenRegistry;
use super::fungible_token::FungibleToken;
use super::management_fund::ManagementFund;
use super::MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME;
use super::storage_key::StorageKey;
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
    /// Registry of investors who are allowed to make an investment deposit.
    investor_account_registry: UnorderedSet<AccountId>,
    current_epoch_height: EpochHeight,
    previous_epoch_rewards_from_validators_near_amount: Balance,       // TODO МОЖет, сделать через ПрошлыйКурс?
    total_rewards_from_validators_near_amount: Balance,       // TODO Все, что связано с ревардс, перенести в структуру?
    /// In bytes.
    storage_usage_per_investor_account: StorageUsage
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
        // TODO Посмотреть, сколько нужно для хранения всего стейта ниже. Остальной депозит пололожить в качестве стейка.

        if env::state_exists() {
            return Err(BaseError::ContractStateAlreadyInitialized);
        }

        if rewards_receiver_account_id == everstake_rewards_receiver_account_id {
            return Err(BaseError::SameAccountId);
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

        let fungible_token_registry = FungibleTokenRegistry {
            classic_token_balance: 0,
            investment_token_balance: 0
        };

        let mut stake_pool = Self {
            owner_id: env::predecessor_account_id(),
            manager_id: manager_id_,
            rewards_receiver_account_id: rewards_receiver_account_id.clone(),
            everstake_rewards_receiver_account_id: everstake_rewards_receiver_account_id.clone(),
            fee_registry: FeeRegistry { rewards_fee, everstake_rewards_fee },
            fungible_token: FungibleToken::new(env::predecessor_account_id())?,
            management_fund: ManagementFund::new()?,
            validating_node: ValidatingNode::new(validators_maximum_quantity)?,
            investor_account_registry: Self::initialize_investor_account_registry(),
            current_epoch_height: env::epoch_height(),
            previous_epoch_rewards_from_validators_near_amount: 0,
            total_rewards_from_validators_near_amount: 0,
            storage_usage_per_investor_account: Self::calculate_storage_usage_per_additional_investor_account()?
        };
        stake_pool.fungible_token.token_account_registry.insert(&rewards_receiver_account_id, &fungible_token_registry);
        stake_pool.fungible_token.token_account_registry.insert(&everstake_rewards_receiver_account_id, &fungible_token_registry);
        stake_pool.fungible_token.token_accounts_quantity = 2;

        Ok(stake_pool)
    }

    fn internal_classic_deposit(&mut self) -> Result<(), BaseError> {
        self.assert_epoch_is_synchronized()?;

        let predecessor_account_id = env::predecessor_account_id();

        let mut near_amount = env::attached_deposit();
        let (mut fungible_token_registry, is_exist): (FungibleTokenRegistry, bool) = match self.fungible_token.token_account_registry.get(&predecessor_account_id) {
            Some(fungible_token_registry_) => (fungible_token_registry_, true),
            None => {
                let storage_staking_price_per_additional_token_account = Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_token_account)?;
                if near_amount < storage_staking_price_per_additional_token_account {
                    return Err(BaseError::InsufficientNearDepositForStorageStaking);
                }
                near_amount -= storage_staking_price_per_additional_token_account;

                (
                    FungibleTokenRegistry {
                        classic_token_balance: 0,
                        investment_token_balance: 0
                    },
                    false
                )
            }
        };
        if near_amount == 0 {
            return Err(BaseError::InsufficientNearDeposit);
        }

        let classic_token_amount = self.convert_near_amount_to_token_amount(near_amount)?;
        if classic_token_amount == 0 {
            return Err(BaseError::InsufficientNearDeposit);
        }

        fungible_token_registry.classic_token_balance += classic_token_amount;

        if self.management_fund.is_distributed_on_validators_in_current_epoch
            && self.validating_node.preffered_validtor_account.is_some() {
                match self.validating_node.preffered_validtor_account {
                    Some(ref preffered_validator_account_id) => {
                        match self.validating_node.validator_account_registry.get(preffered_validator_account_id) {
                            Some(validator_info) => {
                                match validator_info.staking_contract_version {
                                    ValidatorStakingContractVersion::Classic => {
                                        ext_staking_pool::ext(preffered_validator_account_id.clone())
                                            .with_attached_deposit(near_amount)
                                            // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                                            .deposit_and_stake()
                                            .then(
                                                Self::ext(env::current_account_id())
                                                    .classic_deposit_callback(
                                                        &predecessor_account_id,
                                                        &fungible_token_registry,
                                                        preffered_validator_account_id,
                                                        near_amount,
                                                        classic_token_amount,
                                                        is_exist,
                                                        env::epoch_height()
                                                    )
                                            );
                                    }
                                }
                            }
                            None => {
                                return Err(BaseError::Logic);
                            }
                        }
                    }
                    None => {
                        return Err(BaseError::Logic);
                    }
                }
        } else {
            self.management_fund.classic_unstaked_balance += near_amount;
            self.fungible_token.total_supply += classic_token_amount;
            self.fungible_token.token_account_registry.insert(&predecessor_account_id, &fungible_token_registry);
            if !is_exist {
                self.fungible_token.token_accounts_quantity += 1;
            }
        }

        Ok(())
    }

    fn internal_investment_deposit(&mut self) -> Result<(), BaseError> {
        self.assert_epoch_is_synchronized()?;

        let predecessor_account_id = env::predecessor_account_id();

        if !self.investor_account_registry.contains(&predecessor_account_id) {
            return Err(BaseError::InvestorAccountIsNotRegistered);
        }

        let mut near_amount = env::attached_deposit();
        let mut fungible_token_registry: FungibleTokenRegistry = match self.fungible_token.token_account_registry.get(&predecessor_account_id) {
            Some(fungible_token_registry_) => fungible_token_registry_,
            None => {
                let storage_staking_price_per_additional_token_account = Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_token_account)?;
                if near_amount < storage_staking_price_per_additional_token_account {
                    return Err(BaseError::InsufficientNearDepositForStorageStaking);
                }
                near_amount -= storage_staking_price_per_additional_token_account;

                self.fungible_token.token_accounts_quantity += 1;

                FungibleTokenRegistry {
                    classic_token_balance: 0,
                    investment_token_balance: 0
                }
            }
        };
        if near_amount == 0 {
            return Err(BaseError::InsufficientNearDeposit);
        }

        let investment_token_amount = self.convert_near_amount_to_token_amount(near_amount)?;
        if investment_token_amount == 0 {
            return Err(BaseError::InsufficientNearDeposit);
        }

        fungible_token_registry.investment_token_balance += investment_token_amount;

        self.management_fund.investment_unstaked_balance += near_amount;
        self.fungible_token.total_supply += investment_token_amount;
        self.fungible_token.token_account_registry.insert(&predecessor_account_id, &fungible_token_registry);

        Ok(())
    }

    fn internal_classic_instant_withdraw(&mut self, classic_token_amount: u128) -> Result<Promise, BaseError> {   // TODO проставить процент на снятие!!
        self.assert_epoch_is_synchronized()?;

        let predecessor_account_id = env::predecessor_account_id();

        if classic_token_amount == 0 {
            return Err(BaseError::InsufficientTokenDeposit);
        }

        let mut fungible_token_registry = match self.fungible_token.token_account_registry.get(&predecessor_account_id) {
            Some(fungible_token_registry_) => fungible_token_registry_,
            None => {
                return Err(BaseError::TokenAccountIsNotRegistered);
            }
        };
        if fungible_token_registry.classic_token_balance < classic_token_amount {
            return Err(BaseError::InsufficientTokenAccountBalance);
        }

        let mut near_amount = self.convert_token_amount_to_near_amount(classic_token_amount)?;
        if near_amount == 0 {
            return Err(BaseError::InsufficientTokenDeposit);
        }
        if near_amount > self.management_fund.classic_unstaked_balance {
            return Err(BaseError::InsufficientAvailableForStakingBalance);
        }
        self.management_fund.classic_unstaked_balance -= near_amount;

        fungible_token_registry.classic_token_balance -= classic_token_amount;
        if fungible_token_registry.classic_token_balance > 0
            || predecessor_account_id == self.rewards_receiver_account_id
            || predecessor_account_id == self.everstake_rewards_receiver_account_id  {
            self.fungible_token.token_account_registry.insert(&predecessor_account_id, &fungible_token_registry);
        } else {
            if let None = self.fungible_token.token_account_registry.remove(&predecessor_account_id) {
                return Err(BaseError::Logic);
            }
            self.fungible_token.token_accounts_quantity -= 1;

            near_amount += Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_token_account)?;
        }

        self.fungible_token.total_supply -= classic_token_amount;

        Ok(
            Promise::new(predecessor_account_id)
                .transfer(near_amount)
        )
    }

    fn internal_investment_instant_withdraw(&mut self, investment_token_amount: u128) -> Result<Promise, BaseError> {
        todo!();
        // self.assert_epoch_is_synchronized()?;

        // let predecessor_account_id = env::predecessor_account_id();

        // if investment_token_amount == 0 {
        //     return Err(BaseError::InsufficientTokenDeposit);
        // }

        // let mut fungible_token_registry = match self.fungible_token.token_account_registry.get(&predecessor_account_id) {
        //     Some(fungible_token_registry_) => fungible_token_registry_,
        //     None => {
        //         return Err(BaseError::TokenAccountIsNotRegistered);
        //     }
        // };
        // if fungible_token_registry.investment_token_balance < investment_token_amount {
        //     return Err(BaseError::InsufficientTokenAccountBalance);
        // }

        // let mut near_amount = self.convert_token_amount_to_near_amount(investment_token_amount)?;
        // if near_amount == 0 {
        //     return Err(BaseError::InsufficientTokenDeposit);
        // }
        // if near_amount > self.management_fund.investment_unstaked_balance {
        //     return Err(BaseError::InsufficientAvailableForStakingBalance);
        // }
        // self.management_fund.investment_unstaked_balance -= near_amount;

        // fungible_token_registry.investment_token_balance -= investment_token_amount;
        // if fungible_token_registry.investment_token_balance > 0
        //     || predecessor_account_id == self.rewards_receiver_account_id
        //     || predecessor_account_id == self.everstake_rewards_receiver_account_id  {
        //     self.fungible_token.token_account_registry.insert(&predecessor_account_id, &fungible_token_registry);
        // } else {
        //     if let None = self.fungible_token.token_account_registry.remove(&predecessor_account_id) {
        //         return Err(BaseError::Logic);
        //     }
        //     self.fungible_token.token_accounts_quantity -= 1;

        //     near_amount += Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_token_account)?;
        // }

        // self.fungible_token.total_supply -= investment_token_amount;

        // Ok(
        //     Promise::new(predecessor_account_id)
        //         .transfer(near_amount)
        // )
    }

    fn internal_classic_delayed_withdraw(&mut self, classic_token_amount: u128) -> Result<(), BaseError> {
        self.assert_epoch_is_synchronized()?;

        let predecessor_account_id = env::predecessor_account_id();

        let mut near_refundable_deposit = match self.management_fund.delayed_withdrawal_account_registry.get(&predecessor_account_id) {
            Some(_) => {
                return Err(BaseError::DelayedWithdrawalAccountAlreadyRegistered);
            }
            None => {
                let near_deposit = env::attached_deposit();

                let storage_staking_price_per_additional_delayed_withdrawal_account = Self::calculate_storage_staking_price(self.management_fund.storage_usage_per_delayed_withdrawal_account)?;
                if near_deposit < storage_staking_price_per_additional_delayed_withdrawal_account {
                    return Err(BaseError::InsufficientNearDeposit);
                }

                near_deposit - storage_staking_price_per_additional_delayed_withdrawal_account
            }
        };

        if classic_token_amount == 0 {
            return Err(BaseError::InsufficientTokenDeposit);
        }

        let mut fungible_token_registry = match self.fungible_token.token_account_registry.get(&predecessor_account_id) {
            Some(fungible_token_registry_) => fungible_token_registry_,
            None => {
                return Err(BaseError::TokenAccountIsNotRegistered);
            }
        };
        if fungible_token_registry.classic_token_balance < classic_token_amount {
            return Err(BaseError::InsufficientTokenAccountBalance);
        }

        let near_amount = self.convert_token_amount_to_near_amount(classic_token_amount)?;
        if near_amount == 0 {
            return Err(BaseError::InsufficientTokenDeposit);
        }
        if near_amount > self.management_fund.classic_staked_balance {
            return Err(BaseError::InsufficientStakedBalance);
        }

        self.management_fund.classic_staked_balance -= near_amount;
        self.management_fund.delayed_withdrawal_account_registry.insert(
            &predecessor_account_id,
            &DelayedWithdrawalInfo {
                requested_near_amount: near_amount,
                received_near_amount: 0,
                started_epoch_height: env::epoch_height()
            }
        );

        fungible_token_registry.classic_token_balance -= classic_token_amount;
        if fungible_token_registry.classic_token_balance > 0
            || predecessor_account_id == self.rewards_receiver_account_id
            || predecessor_account_id == self.everstake_rewards_receiver_account_id  {
            self.fungible_token.token_account_registry.insert(&predecessor_account_id, &fungible_token_registry);
        } else {
            if let None = self.fungible_token.token_account_registry.remove(&predecessor_account_id) {
                return Err(BaseError::Logic);
            }
            self.fungible_token.token_accounts_quantity -= 1;

            near_refundable_deposit += Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_token_account)?;
        }

        self.fungible_token.total_supply -= classic_token_amount;

        if near_refundable_deposit > 0 {
            Promise::new(predecessor_account_id)
                .transfer(near_refundable_deposit);
        }

        Ok(())
    }

    fn internal_take_delayed_withdrawal(&mut self, delayed_withdrawal_account_id: AccountId) -> Result<Promise, BaseError> {
        self.assert_epoch_is_synchronized()?;

        match self.management_fund.delayed_withdrawal_account_registry.remove(&delayed_withdrawal_account_id) {
            Some(delayed_withdrawal_info) => {
                if delayed_withdrawal_info.requested_near_amount != delayed_withdrawal_info.received_near_amount {
                    return Err(BaseError::Logic);
                }
                if (self.current_epoch_height - delayed_withdrawal_info.started_epoch_height) < EPOCH_QUANTITY_TO_DELAYED_WITHDRAWAL {
                    return Err(BaseError::BadEpoch);
                }
                if delayed_withdrawal_info.received_near_amount > self.management_fund.delayed_withdrawal_balance {
                    return Err(BaseError::Logic);
                }

                self.management_fund.delayed_withdrawal_balance -= delayed_withdrawal_info.received_near_amount;

                let near_amount = delayed_withdrawal_info.received_near_amount + Self::calculate_storage_staking_price(self.management_fund.storage_usage_per_delayed_withdrawal_account)?;

                Ok(
                    Promise::new(env::predecessor_account_id())
                        .transfer(near_amount)
                )
            }
            None => {
                return Err(BaseError::DelayedWithdrawalAccountIsNotRegistered);
            }
        }
    }

    fn internal_add_validator(
        &mut self,
        validator_account_id: AccountId,
        validator_staking_contract_version: ValidatorStakingContractVersion,
        is_preferred: bool
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
            &validator_account_id, &ValidatorInfo::new(validator_staking_contract_version)
        ) {
            return Err(BaseError::ValidatorAccountIsAlreadyRegistered);
        }
        self.validating_node.validator_accounts_quantity += 1;
        self.validating_node.quantity_of_validators_accounts_updated_in_current_epoch += 1;     // TODO вот это точно ли нужно

        if is_preferred {
            self.validating_node.preffered_validtor_account = Some(validator_account_id);
        }

        let near_amount = env::attached_deposit() - storage_staking_price_per_additional_validator_account;
        if near_amount > 0 {
            Promise::new(env::predecessor_account_id())
                .transfer(near_amount);   // TODO Нужен ли коллбек?
        }

        Ok(())
    }

    fn internal_remove_validator(&mut self, validator_account_id: AccountId) -> Result<Promise, BaseError> {
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        match self.validating_node.validator_account_registry.remove(&validator_account_id) {
            Some(validator_info) => {
                if validator_info.classic_staked_balance > 0
                    || validator_info.investment_staked_balance > 0
                    || validator_info.unstaked_balance > 0 {       // TODO  TODO TODO TODO TODO подумать, при каких условиях еще невозможно удалить валидатор.
                    return Err(BaseError::RemovingValidatorWithExistingBalance);
                }
            }
            None => {
                return Err(BaseError::ValidatorAccountIsNotRegistered);
            }
        }

        self.validating_node.validator_accounts_quantity -= 1;
        self.validating_node.quantity_of_validators_accounts_updated_in_current_epoch -= 1;    // TODO  вот это точно ли нужно относительно internal_add_validator

        if let Some(ref preffered_validator_account_id) = self.validating_node.preffered_validtor_account {
            if *preffered_validator_account_id == validator_account_id {
                self.validating_node.preffered_validtor_account = None;
            }
        }

        let near_amount = Self::calculate_storage_staking_price(self.validating_node.storage_usage_per_validator_account)?;

        Ok(
            Promise::new(env::predecessor_account_id())
                .transfer(near_amount)
        )
    }

    fn internal_add_investor(&mut self, investor_account_id: AccountId) -> Result<(), BaseError> {
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        let storage_staking_price_per_additional_investor_account = Self::calculate_storage_staking_price(self.storage_usage_per_investor_account)?;
        if env::attached_deposit() < storage_staking_price_per_additional_investor_account {
            return Err(BaseError::InsufficientNearDepositForStorageStaking);
        }

        if !self.investor_account_registry.insert(&investor_account_id) {
            return Err(BaseError::InvestorAccountIsAlreadyRegistered);
        }

        let near_amount = env::attached_deposit() - storage_staking_price_per_additional_investor_account;
        if near_amount > 0 {
            Promise::new(env::predecessor_account_id())
                .transfer(near_amount);
        }

        Ok(())
    }

    fn internal_remove_investor(&mut self, investor_account_id: AccountId) -> Result<Promise, BaseError> {
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        if let Some(fungible_token_registry) = self.fungible_token.token_account_registry.get(&investor_account_id) {
            if fungible_token_registry.investment_token_balance > 0 {
                return Err(BaseError::RemovingInvestorWithExistingBalance);
            }
        }

        if !self.investor_account_registry.remove(&investor_account_id) {
            return Err(BaseError::InvestorAccountIsNotRegistered);
        }

        let near_amount = Self::calculate_storage_staking_price(self.storage_usage_per_investor_account)?;

        Ok(
            Promise::new(env::predecessor_account_id())
                .transfer(near_amount)
        )
    }

    fn internal_increase_validator_classic_stake(
        &mut self,
        validator_account_id: AccountId,
        near_amount: Balance
    ) -> Result<Promise, BaseError> {      // TODO Сюда нужно зафиксировать максимальное число Газа. Возможно ли?
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        if self.management_fund.classic_unstaked_balance == 0
            || !(1..=self.management_fund.classic_unstaked_balance).contains(&near_amount) {
            return Err(BaseError::InsufficientAvailableForStakingBalance);
        }

        // let deposit_and_stake_gas = Gas(ONE_TERA * Self::DEPOSIT_AND_STAKE_TGAS);           // TODO проверка, сколько газа прикрепили

        match self.validating_node.validator_account_registry.get(&validator_account_id) {
            Some(validator_info) => {
                match validator_info.staking_contract_version {
                    ValidatorStakingContractVersion::Classic => {
                        return Ok(
                            ext_staking_pool::ext(validator_account_id.clone())
                                .with_attached_deposit(near_amount)
                                // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                                .deposit_and_stake()
                                .then(
                                    Self::ext(env::current_account_id())
                                        .increase_validator_classic_stake_callback(
                                            &validator_account_id, near_amount, env::epoch_height()
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

    fn internal_decrease_validator_classic_stake(
        &mut self,
        validator_account_id: AccountId,
        delayed_withdrawal_account_id: AccountId,
        near_amount: Balance
    ) -> Result<Promise, BaseError> {      // TODO Сюда нужно зафиксировать максимальное число Газа. Возможно ли?
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        if self.current_epoch_height % 4 != 0  {
            return Err(BaseError::NotRightEpoch);
        }

        if near_amount == 0 {
            return Err(BaseError::InsufficientNearDeposit);
        }

        match self.management_fund.delayed_withdrawal_account_registry.get(&delayed_withdrawal_account_id) {
            Some(delayed_withdrawal_info) => {
                if (near_amount + delayed_withdrawal_info.received_near_amount) > delayed_withdrawal_info.requested_near_amount {
                    return Err(BaseError::InsufficientDelayedWithdrawalAmount);
                }
            }
            None => {
                return Err(BaseError::DelayedWithdrawalAccountIsNotRegistered);
            }
        }
// TODO проверить анстейкед и стейкед баланс на валидаторах и их запросы отсюда.
        match self.validating_node.validator_account_registry.get(&validator_account_id) {
            Some(validator_info) => {
                if near_amount > validator_info.classic_staked_balance {
                    return Err(BaseError::InsufficientStakedBalance);
                }

                match validator_info.staking_contract_version {
                    ValidatorStakingContractVersion::Classic => {
                        return Ok(
                            ext_staking_pool::ext(validator_account_id.clone())
                                // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                                .unstake(near_amount.into())
                                .then(
                                    Self::ext(env::current_account_id())
                                        .decrease_validator_classic_stake_callback(&validator_account_id, &delayed_withdrawal_account_id, near_amount)
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

    fn internal_take_unstaked_balance(&mut self, validator_account_id: AccountId) -> Result<Promise, BaseError> {      // TODO Сюда нужно зафиксировать максимальное число Газа. Возможно ли?
        self.assert_epoch_is_desynchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        let current_epoch_height = env::epoch_height();

        if current_epoch_height % 4 != 0  {
            return Err(BaseError::NotRightEpoch);
        }

        match self.validating_node.validator_account_registry.get(&validator_account_id) {   // TODO // TODO ЧТо будет, если валидатор перестал работать, что придет с контракта. Не прервется ли из-за этго цепочка выполнения апдейтов
            Some(validator_info) => {
                if validator_info.unstaked_balance == 0 {
                    return Err(BaseError::InsufficientUnstakedBalanceOnValidator);
                }
                if validator_info.last_update_info_epoch_height >= current_epoch_height {
                    return Err(BaseError::ValidatorInfoShouldNotBeUpdated);
                }

                match validator_info.staking_contract_version {
                    ValidatorStakingContractVersion::Classic => {
                        return Ok(
                            ext_staking_pool::ext(validator_account_id.clone())
                                // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                                .withdraw(validator_info.unstaked_balance.into())
                                .then(
                                    Self::ext(env::current_account_id())
                                        .take_unstaked_balance_callback(&validator_account_id)
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
                                    .get_account_staked_balance(env::current_account_id())
                                    .then(
                                        Self::ext(env::current_account_id())
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

        let previous_epoch_rewards_from_validators_classic_token_amount = self.convert_near_amount_to_token_amount(
            self.previous_epoch_rewards_from_validators_near_amount
        )?;

        self.management_fund.classic_staked_balance += self.previous_epoch_rewards_from_validators_near_amount;
        self.management_fund.is_distributed_on_validators_in_current_epoch = false;
        self.validating_node.quantity_of_validators_accounts_updated_in_current_epoch = 0;
        self.current_epoch_height = env::epoch_height();
        self.total_rewards_from_validators_near_amount += self.previous_epoch_rewards_from_validators_near_amount;
        self.previous_epoch_rewards_from_validators_near_amount = 0;

        if let Some(ref rewards_fee) = self.fee_registry.rewards_fee {
            let rewards_fee_classic_token_amount = rewards_fee.multiply(previous_epoch_rewards_from_validators_classic_token_amount);
            if rewards_fee_classic_token_amount != 0 {
                match self.fungible_token.token_account_registry.get(&self.rewards_receiver_account_id) {
                    Some(mut fungible_token_registry) => {
                        fungible_token_registry.classic_token_balance += rewards_fee_classic_token_amount;

                        self.fungible_token.total_supply += rewards_fee_classic_token_amount;
                        self.fungible_token.token_account_registry.insert(&self.rewards_receiver_account_id, &fungible_token_registry);
                    }
                    None => {
                        return Err(BaseError::Logic);
                    }
                }
            }

            if let Some(ref everstake_rewards_fee) = self.fee_registry.everstake_rewards_fee {
                let everstake_rewards_fee_classic_token_amount = everstake_rewards_fee.multiply(rewards_fee_classic_token_amount);
                if everstake_rewards_fee_classic_token_amount != 0 {
                    match self.fungible_token.token_account_registry.get(&self.everstake_rewards_receiver_account_id) {
                        Some(mut fungible_token_registry) => {
                            fungible_token_registry.classic_token_balance += everstake_rewards_fee_classic_token_amount;

                            self.fungible_token.total_supply += everstake_rewards_fee_classic_token_amount;
                            self.fungible_token.token_account_registry.insert(&self.everstake_rewards_receiver_account_id, &fungible_token_registry);
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

    fn internal_change_preffered_validator(&mut self, validator_account_id: Option<AccountId>) -> Result<(), BaseError> {
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        match validator_account_id {
            Some(validator_account_id_) => {
                match self.validating_node.validator_account_registry.get(&validator_account_id_) {
                    Some(_) => {
                        self.validating_node.preffered_validtor_account = Some(validator_account_id_);
                    }
                    None => {
                        return Err(BaseError::ValidatorAccountIsNotRegistered);
                    }
                }
            }
            None => {
                self.validating_node.preffered_validtor_account = None;
            }
        }

        Ok(())
    }

    fn internal_confirm_stake_distribution(&mut self) -> Result<(), BaseError> {
        self.assert_epoch_is_synchronized()?;
        self.assert_authorized_management_only_by_manager()?;

        self.management_fund.is_distributed_on_validators_in_current_epoch = true;

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

    fn internal_get_token_amount_from_near_amount(&self, near_amount: Balance) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        self.convert_near_amount_to_token_amount(near_amount)
    }

    fn internal_get_near_amount_from_token_amount(&self, token_amount: Balance) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        self.convert_token_amount_to_near_amount(token_amount)
    }

    fn internal_get_token_account_balance(&self, account_id: AccountId) -> Result<FungibleTokenRegistryDto, BaseError> {
        match self.fungible_token.token_account_registry.get(&account_id) {
            Some(fungible_token_registry) => {
                let FungibleTokenRegistry {
                    classic_token_balance,
                    investment_token_balance
                } = fungible_token_registry;

                let fungible_token_registry_dto = FungibleTokenRegistryDto {
                    classic_token_balance: classic_token_balance.into(),
                    investment_token_balance: investment_token_balance.into()
                };

                Ok(fungible_token_registry_dto)
            }
            None => {
                return Err(BaseError::TokenAccountIsNotRegistered);
            }
        }
    }

    fn internal_get_unstaked_balance(&self) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(self.management_fund.classic_unstaked_balance + self.management_fund.investment_unstaked_balance)
    }

    fn internal_get_staked_balance(&self) -> Result<Balance, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(self.management_fund.classic_staked_balance + self.management_fund.investment_staked_balance)
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
                staking_contract_version: _,
                unstaked_balance: _,
                classic_staked_balance,
                investment_staked_balance: _,
                last_update_info_epoch_height,
                last_classic_stake_increasing_epoch_height: last_stake_increasing_epoch_height
            } = validator_info;

            validator_info_dto_registry.push(
                ValidatorInfoDto {
                    account_id,
                    classic_staked_balance: classic_staked_balance.into(),
                    last_update_info_epoch_height,
                    last_stake_increasing_epoch_height
                }
            );
        }

        validator_info_dto_registry
    }

    fn internal_get_delayed_withdrawal_info_dto(&self) -> Result<Vec<DelayedWithdrawalInfoDto>, BaseError> {
        self.assert_epoch_is_synchronized()?;

        let mut delayed_withdrawal_info_dto_registry: Vec<DelayedWithdrawalInfoDto> = vec![];

        for (account_id, delayed_withdrawal_info) in self.management_fund.delayed_withdrawal_account_registry.into_iter() {
            let DelayedWithdrawalInfo {
                requested_near_amount,
                received_near_amount,
                started_epoch_height
            } = delayed_withdrawal_info;

            delayed_withdrawal_info_dto_registry.push(
                DelayedWithdrawalInfoDto {
                    account_id,
                    requested_near_amount: requested_near_amount.into(),
                    received_near_amount: received_near_amount.into(),
                    started_epoch_height
                }
            );
        }

        Ok(delayed_withdrawal_info_dto_registry)
    }

    fn internal_get_aggregated_information_dto(&self) -> Result<AggregatedInformationDto, BaseError> {
        self.assert_epoch_is_synchronized()?;

        Ok(
            AggregatedInformationDto {
                unstaked_balance: (self.management_fund.classic_unstaked_balance + self.management_fund.investment_unstaked_balance).into(),
                staked_balance: (self.management_fund.classic_staked_balance + self.management_fund.investment_staked_balance).into(),
                token_total_supply: self.fungible_token.total_supply.into(),
                token_accounts_quantity: self.fungible_token.token_accounts_quantity,
                total_rewards_from_validators_near_amount: self.total_rewards_from_validators_near_amount.into(),
                rewards_fee: self.fee_registry.rewards_fee.clone()
            }
        )
    }

    fn convert_near_amount_to_token_amount(&self, near_amount: Balance) -> Result<Balance, BaseError> {
        if self.management_fund.get_management_fund_amount() == 0 {
            return Ok(near_amount);
        }

        Ok(                  // TODO Проверить Округление
            (
                U256::from(near_amount)
                * U256::from(self.fungible_token.total_supply)
                / U256::from(self.management_fund.get_management_fund_amount())
            ).as_u128()
        )
    }

    fn convert_token_amount_to_near_amount(&self, token_amount: Balance) -> Result<Balance, BaseError> {
        if self.fungible_token.total_supply == 0 {
            return Ok(token_amount);
        }

        Ok(         // TODO Проверить Округление
            (
                U256::from(token_amount)
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

    fn calculate_storage_usage_per_additional_investor_account() -> Result<StorageUsage, BaseError> {
        let mut investor_account_registry = Self::initialize_investor_account_registry();

        let initial_storage_usage = env::storage_usage();

        let account_id = AccountId::new_unchecked("a".repeat(MAXIMIN_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        investor_account_registry.insert(&account_id);

        if env::storage_usage() < initial_storage_usage {
            return Err(BaseError::Logic);
        }

        Ok(env::storage_usage() - initial_storage_usage)
    }

    fn initialize_investor_account_registry() -> UnorderedSet<AccountId> {
        UnorderedSet::new(StorageKey::InvestorAccountRegistry)
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

    /// Stake process with receiving of classic part of fungible token.
    #[payable]
    pub fn classic_deposit(&mut self) {
        if let Err(error) = self.internal_classic_deposit() {
            env::panic_str(format!("{}", error).as_str());
        }
    }

    /// Stake process with receiving of investment part of fungible token.
    #[payable]
    pub fn investment_deposit(&mut self) {
        if let Err(error) = self.internal_investment_deposit() {
            env::panic_str(format!("{}", error).as_str());
        }
    }

    /// Instant unstake process with sending of classic part of fungible token.
    pub fn classic_instant_withdraw(&mut self, classic_token_amount: U128) -> Promise {
        match self.internal_classic_instant_withdraw(classic_token_amount.into()) {
            Ok(promise) => {
                promise
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    /// Instant unstake process with sending of investment part of fungible token.
    pub fn investment_instant_withdraw(&mut self, investment_token_amount: U128) -> Promise {
        match self.internal_investment_instant_withdraw(investment_token_amount.into()) {
            Ok(promise) => {
                promise
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    /// Delayed unstake process with sending of classic part of fungible token.
    #[payable]
    pub fn classic_delayed_withdraw(&mut self, classic_token_amount: U128) {
        if let Err(error) = self.internal_classic_delayed_withdraw(classic_token_amount.into()) {
            env::panic_str(format!("{}", error).as_str());
        }
    }

    pub fn take_delayed_withdrawal(&mut self, delayed_withdrawal_account_id: AccountId) -> Promise {
        match self.internal_take_delayed_withdrawal(delayed_withdrawal_account_id) {
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
        is_preferred: bool
    ) {
        if let Err(error) = self.internal_add_validator(validator_account_id, validator_staking_contract_version, is_preferred) {
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

    #[payable]
    pub fn add_investor(&mut self, investor_account_id: AccountId) {
        if let Err(error) = self.internal_add_investor(investor_account_id) {
            env::panic_str(format!("{}", error).as_str());
        }
    }

    pub fn remove_investor(&mut self, investor_account_id: AccountId) -> Promise {
        match self.internal_remove_investor(investor_account_id) {
            Ok(promise) => {
                promise
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn increase_validator_classic_stake(
        &mut self,
        validator_account_id: AccountId,
        near_amount: Balance
    ) -> Promise {
        match self.internal_increase_validator_classic_stake(validator_account_id, near_amount) {
            Ok(promise) => {
                promise
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn decrease_validator_classic_stake(
        &mut self,
        validator_account_id: AccountId,
        delayed_withdrawal_account_id: AccountId,
        near_amount: Balance
    ) -> Promise {
        match self.internal_decrease_validator_classic_stake(validator_account_id, delayed_withdrawal_account_id, near_amount) {
            Ok(promise) => {
                promise
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn take_unstaked_balance(&mut self, validator_account_id: AccountId) -> Promise {
        match self.internal_take_unstaked_balance(validator_account_id) {
            Ok(promise) => {
                promise
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn update_validator_info(&mut self, validator_account_id: AccountId) -> Promise {
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

    pub fn change_preffered_validator(&mut self, validator_account_id: Option<AccountId>) {
        if let Err(error) = self.internal_change_preffered_validator(validator_account_id) {
            env::panic_str(format!("{}", error).as_str());
        }
    }

    pub fn confirm_stake_distribution(&mut self) {
        if let Err(error) = self.internal_confirm_stake_distribution() {
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

    pub fn get_token_amount_from_near_amount(&self, near_amount: Balance) -> U128 {
        match self.internal_get_token_amount_from_near_amount(near_amount) {
            Ok(token_amount) => {
                token_amount.into()
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn get_near_amount_from_token_amount(&self, token_amount: Balance) -> U128 {
        match self.internal_get_near_amount_from_token_amount(token_amount) {
            Ok(near_amount) => {
                near_amount.into()
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn get_token_account_balance(&self, account_id: AccountId) -> FungibleTokenRegistryDto {
        match self.internal_get_token_account_balance(account_id) {
            Ok(fungible_token_registry_dto) => {
                fungible_token_registry_dto
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn get_unstaked_balance(&self) -> U128 {
        match self.internal_get_unstaked_balance() {
            Ok(unstaked_balance) => {
                unstaked_balance.into()
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

    pub fn get_delayed_withdrawal_info_dto(&self) -> Vec<DelayedWithdrawalInfoDto> { // TODO есть Info , есть Information (проблема в имени)
        match self.internal_get_delayed_withdrawal_info_dto() {
            Ok(delayed_withdrawal_info_dto) => {
                delayed_withdrawal_info_dto
            }
            Err(error) => {
                env::panic_str(format!("{}", error).as_str());
            }
        }
    }

    pub fn get_aggregated_information_dto(&self) -> AggregatedInformationDto { // TODO есть Info , есть Information (проблема в имени)
        match self.internal_get_aggregated_information_dto() {
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
    pub fn increase_validator_classic_stake_callback(
        &mut self,
        validator_account_id: &AccountId,
        near_amount: Balance,
        current_epoch_height: EpochHeight
    ) -> bool {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");        // TODO Фраза повторяется. Нужно ли выновсить в константу?
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                self.management_fund.classic_unstaked_balance -= near_amount;
                self.management_fund.classic_staked_balance += near_amount;

                let mut validator_info = self.validating_node.validator_account_registry.get(validator_account_id).unwrap();  // TODO unwrap     МОЖНО ПереДАВАТЬ в КОЛЛБЭК этот объектОБЪЕКТ Сразу
                validator_info.classic_staked_balance += near_amount;
                validator_info.last_classic_stake_increasing_epoch_height = Some(current_epoch_height);
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

                let current_staked_balance = validator_info.classic_staked_balance + validator_info.investment_staked_balance;

                let staking_rewards_near_amount = if new_staked_balance >= current_staked_balance {
                    new_staked_balance - current_staked_balance
                } else {
                    env::panic_str("Contract logic error.");        // TODO  как обоработать. Может, возвращать структуры ?
                };

                validator_info.last_update_info_epoch_height = current_epoch_height;
                validator_info.classic_staked_balance = new_staked_balance - validator_info.investment_staked_balance;

                self.validating_node.validator_account_registry.insert(validator_account_id, &validator_info);
                self.validating_node.quantity_of_validators_accounts_updated_in_current_epoch += 1;

                self.previous_epoch_rewards_from_validators_near_amount += staking_rewards_near_amount;

                (true, env::epoch_height())
            }
            _ => {
                (false, env::epoch_height())
            }
        }
    }

    #[private]
    pub fn decrease_validator_classic_stake_callback(
        &mut self,
        validator_account_id: &AccountId,
        delayed_withdrawal_account_id: &AccountId,
        near_amount: Balance
    ) -> bool {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let mut delayed_withdrawal_info = self.management_fund.delayed_withdrawal_account_registry.get(delayed_withdrawal_account_id).unwrap(); // TODO передать объект сразу
                delayed_withdrawal_info.received_near_amount += near_amount;

                let mut validator_info = self.validating_node.validator_account_registry.get(validator_account_id).unwrap(); // TODO передать объект
                validator_info.classic_staked_balance -= near_amount;
                validator_info.unstaked_balance += near_amount;

                self.management_fund.delayed_withdrawal_account_registry.insert(delayed_withdrawal_account_id, &delayed_withdrawal_info);
                self.validating_node.validator_account_registry.insert(validator_account_id, &validator_info);

                true
            }
            _ => {
                false
            }
        }
    }

    #[private]
    pub fn take_unstaked_balance_callback(&mut self, validator_account_id: &AccountId) -> bool {            // TODO Может быть, ставить счетчик на количество валиаторов, с которыз нужно снимать стейк, чтобы проверять.
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let mut validator_info = self.validating_node.validator_account_registry.get(validator_account_id).unwrap(); // TODO передать объект

                self.management_fund.delayed_withdrawal_balance += validator_info.unstaked_balance;

                validator_info.unstaked_balance = 0;

                self.validating_node.validator_account_registry.insert(validator_account_id, &validator_info);

                true
            }
            _ => {
                false
            }
        }
    }

    #[private]
    pub fn classic_deposit_callback(
        &mut self,
        predecessor_account_id: &AccountId,
        fungible_token_registry: &FungibleTokenRegistry,
        validator_account_id: &AccountId,
        near_amount: Balance,
        classic_token_amount: Balance,
        is_existing_token_account: bool,
        current_epoch_height: EpochHeight
    ) {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");        // TODO Фраза повторяется. Нужно ли выновсить в константу?
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                self.management_fund.classic_staked_balance += near_amount;

                let mut validator_info = self.validating_node.validator_account_registry.get(validator_account_id).unwrap();  // TODO unwrap     МОЖНО ПереДАВАТЬ в КОЛЛБЭК этот объектОБЪЕКТ Сразу
                validator_info.classic_staked_balance += near_amount;
                validator_info.last_classic_stake_increasing_epoch_height = Some(current_epoch_height);
                self.validating_node.validator_account_registry.insert(validator_account_id, &validator_info);
            }
            _ => {
                self.management_fund.classic_unstaked_balance += near_amount;
            }
        }

        self.fungible_token.total_supply += classic_token_amount;
        self.fungible_token.token_account_registry.insert(&predecessor_account_id, &fungible_token_registry);
        if !is_existing_token_account {
            self.fungible_token.token_accounts_quantity += 1;
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




// TODO Реализовать протоколы. Например, обмен токенами между полльзователями. Обратить внимание на ИнвестингТокенс



// TODO TODO TODO TODO TODO ВСе коллбеки сделать так, чтоы приходило БорщСериалайзед данные, а не В Джсоне