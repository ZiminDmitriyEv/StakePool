use core::convert::Into;
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_sdk::{env, near_bindgen, PanicOnDefault, AccountId, Balance, EpochHeight, Promise, PromiseResult, StorageUsage, Gas, PromiseOrValue};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use super::cross_contract_call::validator::validator;
use super::data_transfer_object::aggregated_info::AggregatedInfo;
use super::data_transfer_object::investor_investment_info::InvestorInvestmentInfo as InvestorInvestmentInfoDto;
use super::data_transfer_object::requested_to_withdrawal_fund::RequestedToWithdrawalFund;
use super::data_transfer_object::storage_staking_price::StorageStakingPrice;
use super::data_transfer_object::validator_info::ValidatorInfo as ValidatorInfoDto;
use super::delayed_withdrawal_info::DelayedWithdrawalInfo;
use super::fee_registry::FeeRegistry;
use super::fee::Fee;
use super::fungible_token::FungibleToken;
use super::investment_withdrawal_info::InvestmentWithdrawalInfo;
use super::investor_investment_info::InvestorInvestmentInfo;
use super::management_fund::ManagementFund;
use super::MAXIMUM_NUMBER_OF_TGAS;
use super::MINIMUM_ATTACHED_DEPOSIT;
use super::stake_decreasing_kind::StakeDecreasingType;
use super::validating_node::ValidatingNode;
use super::validator_info::ValidatorInfo;
use super::validator_staking_contract_version::ValidatorStakingContractVersion;
use uint::construct_uint;

construct_uint! {
    pub struct U256(4);
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]     // TODO проверить все типы данных. LazyOption, например, добавить там, где Мэпы и сеты, посмотреть, где нужно !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!1
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
    previous_epoch_rewards_from_validators_near_amount: Balance,       // TODO МОЖет, сделать через ПрошлыйКурс?
    total_rewards_from_validators_near_amount: Balance,       // TODO Все, что связано с ревардс, перенести в структуру?
}

#[near_bindgen]
impl StakePool {
    /// Call-methods:
    #[init]
    pub fn new(
        manager_id: Option<AccountId>,
        rewards_receiver_account_id: AccountId,
        everstake_rewards_receiver_account_id: AccountId,
        rewards_fee: Option<Fee>,
        everstake_rewards_fee: Option<Fee>,
        validators_maximum_quantity: Option<u64>
    ) -> Self {      // TODO Сюда заходит дипозит. Как его отсечь, то есть, взять ту часть, к
        Self::internal_new(
            manager_id,
            rewards_receiver_account_id,
            everstake_rewards_receiver_account_id,
            rewards_fee,
            everstake_rewards_fee,
            validators_maximum_quantity
        )
    }

    /// Stake process.
    #[payable]
    pub fn deposit(&mut self) {
        self.internal_deposit();
    }

    /// Stake process directly to the validator.
    /// Available only for Investor.
    #[payable]
    pub fn deposit_on_validator(&mut self, near_amount: U128, validator_account_id: AccountId) {
        self.internal_deposit_on_validator(near_amount.into(), validator_account_id);
    }

    /// Instant unstake process.
    pub fn instant_withdraw(&mut self, token_amount: U128) -> Promise {
        self.internal_instant_withdraw(token_amount.into())
    }

    /// Delayed unstake process.
    #[payable]
    pub fn delayed_withdraw(&mut self, token_amount: U128) -> PromiseOrValue<()> {
        self.internal_delayed_withdraw(token_amount.into())
    }

    /// Delayed unstake process directly from validator
    /// Available only for Investor.
    #[payable]
    pub fn delayed_withdraw_from_validator(&mut self, near_amount: U128, validator_account_id: AccountId) -> PromiseOrValue<()> {
        self.internal_delayed_withdraw_from_validator(near_amount.into(), validator_account_id)
    }

    pub fn take_delayed_withdrawal(&mut self) -> Promise {
        self.internal_take_delayed_withdrawal()
    }

    pub fn increase_validator_stake(&mut self, validator_account_id: AccountId, near_amount: U128) -> Promise {
        self.internal_increase_validator_stake(validator_account_id, near_amount.into())
    }

    /// Validator stake decreasing process for the needs of delayed withdrawal fund.
    pub fn requested_decrease_validator_stake(
        &mut self,
        validator_account_id: AccountId,
        near_amount: U128,
        stake_decreasing_type: StakeDecreasingType
    ) -> Promise {
        self.internal_requested_decrease_validator_stake(validator_account_id, near_amount.into(), stake_decreasing_type)
    }

    pub fn take_unstaked_balance(&mut self, validator_account_id: AccountId) -> Promise {
        self.internal_take_unstaked_balance(validator_account_id)
    }

    pub fn update_validator_info(&mut self, validator_account_id: AccountId) -> Promise {
        self.internal_update_validator_info(validator_account_id)
    }

    pub fn update(&mut self) {
        self.internal_update();
    }

    #[payable]
    pub fn add_validator(
        &mut self,
        validator_account_id: AccountId,
        validator_staking_contract_version: ValidatorStakingContractVersion,
        is_only_for_investment: bool,
        is_preferred: bool
    ) -> PromiseOrValue<()> {
        self.internal_add_validator(validator_account_id, validator_staking_contract_version, is_only_for_investment, is_preferred)
    }

    pub fn change_validator_investment_context(&mut self, validator_account_id: AccountId, is_only_for_investment: bool) {
        self.internal_change_validator_investment_context(validator_account_id, is_only_for_investment);
    }

    pub fn change_preffered_validator(&mut self, validator_account_id: Option<AccountId>) {
        self.internal_change_preffered_validator(validator_account_id);
    }

    pub fn remove_validator(&mut self, validator_account_id: AccountId) -> Promise {
        self.internal_remove_validator(validator_account_id)
    }

    #[payable]
    pub fn add_investor(&mut self, investor_account_id: AccountId) -> PromiseOrValue<()> {
        self.internal_add_investor(investor_account_id)
    }

    pub fn remove_investor(&mut self, investor_account_id: AccountId) -> Promise {
        self.internal_remove_investor(investor_account_id)
    }

    pub fn change_manager(&mut self, manager_id: AccountId) {
        self.internal_change_manager(manager_id);
    }

    pub fn change_rewards_fee(&mut self, rewards_fee: Option<Fee>) {
        self.internal_change_rewards_fee(rewards_fee);
    }

    pub fn change_everstake_rewards_fee(&mut self, everstake_rewards_fee: Option<Fee>) {
        self.internal_change_everstake_rewards_fee(everstake_rewards_fee);
    }

    pub fn confirm_stake_distribution(&mut self) {
        self.internal_confirm_stake_distribution();
    }

    /// View-methods:

    pub fn has_delayed_withdrawal(&self, account_id: AccountId) -> bool {
        self.internal_has_delayed_withdrawal(account_id)
    }

    pub fn get_delayed_withdrawal_details(&self, account_id: AccountId) -> (u64, U128) {
        self.internal_get_delayed_withdrawal_details(account_id)
    }

    pub fn is_account_registered(&self, account_id: AccountId) -> bool {
        self.internal_is_account_registered(account_id)
    }

    pub fn get_account_balance(&self, account_id: AccountId) -> (U128, U128, U128, U128) {
        self.internal_get_account_balance(account_id)
    }

    pub fn get_total_token_supply(&self) -> U128 {
        self.internal_get_total_token_supply().into()
    }

    pub fn get_stakers_quantity(&self) -> u64 {
        self.internal_get_stakers_quantity()
    }

    pub fn get_storage_staking_price(&self) -> StorageStakingPrice {
        self.internal_get_storage_staking_price()
    }

    pub fn get_management_fund(&self) -> (U128, U128, U128) {
        self.internal_get_management_fund()
    }

    pub fn get_fee_registry(&self) -> FeeRegistry {
        self.internal_get_fee_registry()
    }

    pub fn get_current_epoch_height(&self) -> (EpochHeight, EpochHeight) {
        self.internal_get_current_epoch_height()
    }

    pub fn is_stake_distributed(&self) -> bool {
        self.internal_is_stake_distributed()
    }

    pub fn is_investor(&self, account_id: AccountId) -> bool {
        self.internal_is_investor(account_id)
    }

    pub fn get_investor_investment_info(&self, account_id: AccountId) -> InvestorInvestmentInfoDto {
        self.internal_get_investor_investment_info(account_id)
    }

    pub fn get_validator_info_registry(&self) -> Vec<ValidatorInfoDto> {
        self.internal_get_validator_info_registry()
    }

    pub fn get_aggregated_info(&self) -> AggregatedInfo {
        self.internal_get_aggregated_info()
    }

    pub fn get_requested_to_withdrawal_fund(&self) -> RequestedToWithdrawalFund {
        self.internal_get_requested_to_withdrawal_fund()
    }
}

#[near_bindgen]
impl FungibleTokenCore for StakePool {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, _memo: Option<String>) {
        self.internal_ft_transfer(receiver_id, amount.into());
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        todo!();
    }

    fn ft_total_supply(&self) -> U128 {
        self.internal_ft_total_supply().into()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.internal_ft_balance_of(account_id).into()
    }
}

impl StakePool {        // TODO TODO TODO добавить логи к каждой манипуляции с деньгами или event. Интерфейсы
    fn internal_new(
        manager_id: Option<AccountId>,
        rewards_receiver_account_id: AccountId,
        everstake_rewards_receiver_account_id: AccountId,
        rewards_fee: Option<Fee>,
        everstake_rewards_fee: Option<Fee>,
        validators_maximum_quantity: Option<u64>
    ) -> Self {                                     // TODO Посмотреть, сколько нужно для хранения всего стейта ниже. Остальной депозит пололожить в качестве стейка.
        if env::state_exists() {
            env::panic_str("Contract state is already initialize.");
        }

        if rewards_receiver_account_id == everstake_rewards_receiver_account_id {
            env::panic_str("The rewards receiver account and everstake rewards receiver account can not be the same.");
        }

        if let Some(ref rewards_fee_) = rewards_fee {
            rewards_fee_.assert_valid();
        }
        if let Some(ref everstake_rewards_fee_) = everstake_rewards_fee {
            everstake_rewards_fee_.assert_valid();
        }

        let manager_id_ = match manager_id {
            Some(manager_id__) => {
                manager_id__
            }
            None => {
                env::predecessor_account_id()
            }
        };

        let mut stake_pool = Self {
            owner_id: env::predecessor_account_id(),
            manager_id: manager_id_,
            rewards_receiver_account_id: rewards_receiver_account_id.clone(),
            everstake_rewards_receiver_account_id: everstake_rewards_receiver_account_id.clone(),
            fee_registry: FeeRegistry { rewards_fee, everstake_rewards_fee },
            fungible_token: FungibleToken::new(env::predecessor_account_id()),
            management_fund: ManagementFund::new(),
            validating_node: ValidatingNode::new(validators_maximum_quantity),
            current_epoch_height: env::epoch_height(),
            previous_epoch_rewards_from_validators_near_amount: 0,
            total_rewards_from_validators_near_amount: 0
        };
        stake_pool.fungible_token.account_registry.insert(&rewards_receiver_account_id, &0);
        stake_pool.fungible_token.account_registry.insert(&everstake_rewards_receiver_account_id, &0);
        stake_pool.fungible_token.accounts_quantity = 2;

        stake_pool
    }

    fn internal_deposit(&mut self) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();

        let predecessor_account_id = env::predecessor_account_id();

        let mut near_amount = env::attached_deposit();

        let mut token_balance = match self.fungible_token.account_registry.get(&predecessor_account_id) {
            Some(token_balance_) => token_balance_,
            None => {
                let storage_staking_price_per_additional_account = Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_account);
                if near_amount < storage_staking_price_per_additional_account {
                    env::panic_str("Insufficient near deposit.");
                }
                near_amount -= storage_staking_price_per_additional_account;

                0
            }
        };
        if near_amount == 0 {
            env::panic_str("Insufficient near deposit.");
        }

        let token_amount = self.convert_near_amount_to_token_amount(near_amount);
        if token_amount == 0 {
            env::panic_str("Insufficient near deposit.");
        }

        if self.management_fund.is_distributed_on_validators_in_current_epoch
            && self.validating_node.preffered_validtor.is_some() {
            match self.validating_node.preffered_validtor {
                Some(ref preffered_validator_account_id) => {
                    match self.validating_node.validator_registry.get(preffered_validator_account_id) {
                        Some(validator_info) => {
                            match validator_info.staking_contract_version {
                                ValidatorStakingContractVersion::Classic => {
                                    validator::ext(preffered_validator_account_id.clone())
                                        .with_attached_deposit(near_amount)
                                        // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                                        .deposit_and_stake()
                                        .then(
                                            Self::ext(env::current_account_id())
                                                .deposit_callback(
                                                    predecessor_account_id,
                                                    preffered_validator_account_id.clone(),
                                                    near_amount,
                                                    token_amount,
                                                    env::epoch_height()
                                                )
                                        );
                                }
                            }
                        }
                        None => {
                            env::panic_str("Object should exist.");
                        }
                    }
                }
                None => {
                    env::panic_str("Object should exist.");
                }
            }
        } else {
            token_balance += token_amount;

            self.management_fund.unstaked_balance += near_amount;
            self.fungible_token.total_supply += token_amount;
            if let None = self.fungible_token.account_registry.insert(&predecessor_account_id, &token_balance) {
                self.fungible_token.accounts_quantity += 1;
            }
        }
    }

    fn internal_deposit_on_validator(&mut self, near_amount: Balance, validator_account_id: AccountId) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();

        if near_amount == 0 {
            env::panic_str("Insufficient near amount.");
        }

        let validator_info = match self.validating_node.validator_registry.get(&validator_account_id) {
            Some(validator_info_) => validator_info_,
            None => {
                env::panic_str("Validator account is not registered yet.");
            }
        };

        let predecessor_account_id = env::predecessor_account_id();

        let attached_deposit = env::attached_deposit();

        let mut storage_staking_price_per_additional_accounts: Balance = 0;

        let investor_investment_info = match self.validating_node.investor_investment_registry.get(&predecessor_account_id) {
            Some(investor_investment_info_) => investor_investment_info_,
            None => {
                env::panic_str("Investor account is not registered yet.");
            }
        };
        if let None = investor_investment_info.distribution_registry.get(&validator_account_id) {
            storage_staking_price_per_additional_accounts += Self::calculate_storage_staking_price(self.validating_node.storage_usage_per_distribution);
        }

        if let None = self.fungible_token.account_registry.get(&predecessor_account_id) {
            storage_staking_price_per_additional_accounts += Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_account);
        };


        if attached_deposit <= storage_staking_price_per_additional_accounts {
            env::panic_str("Insufficient near deposit.");
        }
        let available_for_staking_near_amount = attached_deposit - storage_staking_price_per_additional_accounts;

        if near_amount > available_for_staking_near_amount {
            env::panic_str("Insufficient near deposit.");
        }
        let refundable_near_amount = available_for_staking_near_amount - near_amount;

        let token_amount = self.convert_near_amount_to_token_amount(near_amount);
        if token_amount == 0 {
            env::panic_str("Insufficient near deposit.");
        }

        match validator_info.staking_contract_version {
            ValidatorStakingContractVersion::Classic => {
                validator::ext(validator_account_id.clone())
                    .with_attached_deposit(near_amount)
                    // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                    .deposit_and_stake()
                    .then(
                        Self::ext(env::current_account_id())
                            .deposit_on_validator_callback(
                                predecessor_account_id,
                                validator_account_id.clone(),
                                near_amount,
                                attached_deposit,
                                refundable_near_amount,
                                token_amount
                            )
                    );
            }
        }
    }

    fn internal_instant_withdraw(&mut self, token_amount: Balance) -> Promise {   // TODO проставить процент на снятие!!
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();

        if token_amount == 0 {
            env::panic_str("Insufficient token amount.");
        }

        let predecessor_account_id = env::predecessor_account_id();

        let mut token_balance = match self.fungible_token.account_registry.get(&predecessor_account_id) {
            Some(token_balance_) => token_balance_,
            None => {
                env::panic_str("Token account is not registered.");
            }
        };
        if token_balance < token_amount {
            env::panic_str("Token amount exceeded the available token balance.");
        }

        let mut near_amount = self.convert_token_amount_to_near_amount(token_amount);      // TODO  TODO  TODO  TODO  TODO  Может, конвертацию везде нужно считать на коллбеке в контексте лостАпдейта?
        if near_amount == 0 {
            env::panic_str("Insufficient token amount.");
        }
        if near_amount > self.management_fund.unstaked_balance {
            env::panic_str("Token amount exceeded the available unstaked near balance.");
        }
        self.management_fund.unstaked_balance -= near_amount;

        token_balance -= token_amount;
        if let Some(investor_investment_info) = self.validating_node.investor_investment_registry.get(&predecessor_account_id) {
            if self.convert_token_amount_to_near_amount(token_balance) < investor_investment_info.staked_balance {
                env::panic_str("Token amount exceeded the available to instant withdraw token amount.");
            }
        }
        if token_balance > 0
            || predecessor_account_id == self.rewards_receiver_account_id
            || predecessor_account_id == self.everstake_rewards_receiver_account_id  {
            self.fungible_token.account_registry.insert(&predecessor_account_id, &token_balance);
        } else {
            self.fungible_token.account_registry.remove(&predecessor_account_id);
            self.fungible_token.accounts_quantity -= 1;

            near_amount += Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_account);
        }

        self.fungible_token.total_supply -= token_amount;

        Promise::new(predecessor_account_id)                // TODO вот тут не лучше ли сделать через коллбек
            .transfer(near_amount)
    }

    fn internal_delayed_withdraw(&mut self, token_amount: Balance) -> PromiseOrValue<()> {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();

        if token_amount == 0 {
            env::panic_str("Insufficient token amount.");
        }

        let predecessor_account_id = env::predecessor_account_id();

        let mut token_balance = match self.fungible_token.account_registry.get(&predecessor_account_id) {
            Some(token_balance_) => token_balance_,
            None => {
                env::panic_str("Token account is not registered.");
            }
        };
        if token_balance < token_amount {
            env::panic_str("Token amount exceeded the available token balance.");
        }

        let near_amount = self.convert_token_amount_to_near_amount(token_amount);
        if near_amount == 0 {
            env::panic_str("Insufficient token amount.");
        }

        if near_amount > self.management_fund.staked_balance {
            env::panic_str("Token amount exceeded the available staked near balance.");
        }

        self.management_fund.staked_balance -= near_amount;
        let mut near_refundable_deposit = match self.management_fund.delayed_withdrawn_fund.account_registry.insert(
            &predecessor_account_id,
            &DelayedWithdrawalInfo {
                near_amount,
                started_epoch_height: env::epoch_height()
            }
        ) {
            Some(_) => {
                env::panic_str("Delayed withdrawal account is already registered.");
            }
            None => {
                let near_deposit = env::attached_deposit();

                let storage_staking_price_per_additional_delayed_withdrawal =
                    Self::calculate_storage_staking_price(self.management_fund.delayed_withdrawn_fund.storage_usage_per_account);
                if near_deposit < storage_staking_price_per_additional_delayed_withdrawal {
                    env::panic_str("Insufficient near deposit.");
                }

                near_deposit - storage_staking_price_per_additional_delayed_withdrawal
            }
        };
        self.management_fund.delayed_withdrawn_fund.needed_to_request_classic_near_amount += near_amount;

        token_balance -= token_amount;
        if let Some(investor_investment_info) = self.validating_node.investor_investment_registry.get(&predecessor_account_id) {
            if self.convert_token_amount_to_near_amount(token_balance) < investor_investment_info.staked_balance {
                env::panic_str("Token amount exceeded the available to delayed withdraw token amount.");
            }
        }
        if token_balance > 0
            || predecessor_account_id == self.rewards_receiver_account_id
            || predecessor_account_id == self.everstake_rewards_receiver_account_id  {
            self.fungible_token.account_registry.insert(&predecessor_account_id, &token_balance);
        } else {
           self.fungible_token.account_registry.remove(&predecessor_account_id);
            self.fungible_token.accounts_quantity -= 1;

            near_refundable_deposit += Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_account);
        }

        self.fungible_token.total_supply -= token_amount;

        if near_refundable_deposit > 0 {
            return PromiseOrValue::Promise(
                Promise::new(predecessor_account_id)
                    .transfer(near_refundable_deposit)
            );
        }

        PromiseOrValue::Value(())
    }

    fn internal_delayed_withdraw_from_validator(&mut self, near_amount: Balance, validator_account_id: AccountId) -> PromiseOrValue<()> {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();

        if near_amount == 0 {
            env::panic_str("Insufficient near amount.");
        }
        if near_amount > self.management_fund.staked_balance {
            env::panic_str("Token amount exceeded the available staked near balance.");
        }

        let validator_info = match self.validating_node.validator_registry.get(&validator_account_id) {
            Some(validator_info_) => validator_info_,
            None => {
                env::panic_str("Validator account is not registered yet.");
            }
        };

        let predecessor_account_id = env::predecessor_account_id();

        let mut investor_investment_info = match self.validating_node.investor_investment_registry.get(&predecessor_account_id) {
            Some(investor_investment_info_) => investor_investment_info_,
            None => {
                env::panic_str("Investor account is not registered yet.");
            }
        };

        let mut staked_balance = match investor_investment_info.distribution_registry.get(&validator_account_id) {
            Some(staked_balance_) => staked_balance_,
            None => {
                env::panic_str("There is no investor stake on this validator.");
            }
        };
        if near_amount > staked_balance {
            env::panic_str("Near amount exceeded the available investor near balance on validator.");
        }

        let (mut near_refundable_deposit, mut investment_withdrawal_info) =
            match self.management_fund.delayed_withdrawn_fund.investment_withdrawal_registry.get(&validator_account_id) {
            Some(investment_withdrawal_info_) => (env::attached_deposit(), investment_withdrawal_info_),
            None => {
                let near_deposit = env::attached_deposit();

                let storage_staking_price_per_additional_investment_withdrawal =
                    Self::calculate_storage_staking_price(self.management_fund.delayed_withdrawn_fund.storage_usage_per_investment_withdrawal);
                if near_deposit < storage_staking_price_per_additional_investment_withdrawal {
                    env::panic_str("Insufficient near deposit.");
                }

                (
                    near_deposit - storage_staking_price_per_additional_investment_withdrawal,
                    InvestmentWithdrawalInfo {
                        near_amount: 0,
                        account_id: predecessor_account_id.clone()
                    }
                )
            }
        };
        if near_amount > (validator_info.investment_staked_balance - investment_withdrawal_info.near_amount) {
            env::panic_str("Near amount exceeded the available near balance on validator.");
        }

        let token_amount = self.convert_near_amount_to_token_amount(near_amount);
        if token_amount == 0 {
            env::panic_str("Insufficient near amount.");
        }

        let mut token_balance = match self.fungible_token.account_registry.get(&predecessor_account_id) {  // TODO TODO TODO TODO TODO очень важно делать правильную математику конвертации. То есть, количество токенов округляем в меньшую сторону при конвертации, а количество неаров - в большую. Вообще, нужно, чтобы прямой-обратный перевод работали правильно.
            Some(token_balance_) => token_balance_,
            None => {
                env::panic_str("Token account is not registered.");
            }
        };
        if token_balance < token_amount {
            env::panic_str("Token amount exceeded the available token balance.");
        }

        self.management_fund.staked_balance -= near_amount;
        match self.management_fund.delayed_withdrawn_fund.account_registry.insert(
            &predecessor_account_id,
            &DelayedWithdrawalInfo {
                near_amount,
                started_epoch_height: env::epoch_height()
            }
        ) {
            Some(_) => {
                env::panic_str("Delayed withdrawal account is already registered.");
            }
            None => {
                let storage_staking_price_per_additional_account =
                    Self::calculate_storage_staking_price(self.management_fund.delayed_withdrawn_fund.storage_usage_per_account);
                if near_refundable_deposit < storage_staking_price_per_additional_account {
                    env::panic_str("Insufficient near deposit.");
                }
                near_refundable_deposit -= storage_staking_price_per_additional_account;
            }
        }

        investment_withdrawal_info.near_amount += near_amount;
        self.management_fund.delayed_withdrawn_fund.investment_withdrawal_registry.insert(&validator_account_id, &investment_withdrawal_info);
        self.management_fund.delayed_withdrawn_fund.needed_to_request_investment_near_amount += near_amount;

        if near_amount < staked_balance {
            staked_balance -= near_amount;

            investor_investment_info.distribution_registry.insert(&validator_account_id, &staked_balance);
        } else {
            investor_investment_info.distribution_registry.remove(&validator_account_id);
            investor_investment_info.distributions_quantity -= 1;

            near_refundable_deposit += Self::calculate_storage_staking_price(self.validating_node.storage_usage_per_distribution);
        }
        investor_investment_info.staked_balance -= near_amount;
        self.validating_node.investor_investment_registry.insert(&predecessor_account_id, &investor_investment_info);

        token_balance -= token_amount;
        if token_balance > 0
            || predecessor_account_id == self.rewards_receiver_account_id
            || predecessor_account_id == self.everstake_rewards_receiver_account_id  {
            self.fungible_token.account_registry.insert(&predecessor_account_id, &token_balance);
        } else {
            self.fungible_token.account_registry.remove(&predecessor_account_id);
            self.fungible_token.accounts_quantity -= 1;

            near_refundable_deposit += Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_account);
        }

        self.fungible_token.total_supply -= token_amount;

        if near_refundable_deposit > 0 {
            return PromiseOrValue::Promise(
                Promise::new(predecessor_account_id)
                    .transfer(near_refundable_deposit)
            )
        }

        PromiseOrValue::Value(())
    }

    fn internal_take_delayed_withdrawal(&mut self) -> Promise {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();

        let predecessor_account_id = env::predecessor_account_id();

        match self.management_fund.delayed_withdrawn_fund.account_registry.remove(&predecessor_account_id) {
            Some(delayed_withdrawal_info) => {
                if !delayed_withdrawal_info.can_take_delayed_withdrawal(self.current_epoch_height) {
                    env::panic_str("Wrong epoch for withdrawal.");
                }

                self.management_fund.delayed_withdrawn_fund.balance -= delayed_withdrawal_info.near_amount;

                let near_amount = delayed_withdrawal_info.near_amount +
                    Self::calculate_storage_staking_price(self.management_fund.delayed_withdrawn_fund.storage_usage_per_account);

                Promise::new(predecessor_account_id)
                    .transfer(near_amount)
            }
            None => {
                env::panic_str("Delayed withdrawal account is not registered.");
            }
        }
    }

    fn internal_increase_validator_stake(&mut self, validator_account_id: AccountId, near_amount: Balance) -> Promise {      // TODO Сюда нужно зафиксировать максимальное число Газа. Возможно ли?
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        if near_amount == 0 {
            env::panic_str("Insufficient near amount.");
        }

        if self.management_fund.unstaked_balance == 0
            || !(1..=self.management_fund.unstaked_balance).contains(&near_amount) {
                env::panic_str("Near amount exceeded the available unstaked near balance.");
        }

        match self.validating_node.validator_registry.get(&validator_account_id) {
            Some(validator_info) => {                                                                       // TODO написать все везде в одном стиле let vi = match ... То есть, убрать вложенность там, где возможно
                if validator_info.is_only_for_investment {
                    env::panic_str("Validator is used only for investment purpose.");
                }

                match validator_info.staking_contract_version {
                    ValidatorStakingContractVersion::Classic => {
                        validator::ext(validator_account_id.clone())
                            .with_attached_deposit(near_amount)
                            // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                            .deposit_and_stake()
                            .then(
                                Self::ext(env::current_account_id())
                                    .increase_validator_stake_callback(validator_account_id, near_amount, env::epoch_height())
                            )
                    }
                }
            }
            None => {
                env::panic_str("Validator account is not registered yet.");
            }
        }
    }

    fn internal_requested_decrease_validator_stake(
        &mut self,
        validator_account_id: AccountId,
        near_amount: Balance,
        stake_decreasing_type: StakeDecreasingType
    ) -> Promise {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_desynchronized();
        self.assert_authorized_management_only_by_manager();
        if !Self::epoch_is_right(env::epoch_height()) {
            env::panic_str("Epoch is not intended for a requested decrease validator stake request.");
        }

        if near_amount == 0 {
            env::panic_str("Insufficient near amount.");
        }

        let validator_info = match self.validating_node.validator_registry.get(&validator_account_id) {     // TODO проверить на правильные ли методы идут кроссколы. Взять весб баланс или взять анстейкед баланс или взять стейкед баланс.
            Some(validator_info_) => validator_info_,
            None => {
                env::panic_str("Validator account is not registered yet.");
            }
        };
        match stake_decreasing_type {
            StakeDecreasingType::Classic => {
                if near_amount > validator_info.classic_staked_balance {
                    env::panic_str("Near amount exceeded the available staked near balance.");
                }
                if near_amount > self.management_fund.delayed_withdrawn_fund.needed_to_request_classic_near_amount {
                    env::panic_str("Near amount is more than requested near amount.");
                }
            }
            StakeDecreasingType::Investment => {
                if near_amount > validator_info.investment_staked_balance {
                    env::panic_str("Near amount exceeded the available unstaked near balance.");
                }
                if near_amount > self.management_fund.delayed_withdrawn_fund.needed_to_request_investment_near_amount {
                    env::panic_str("Near amount is more than requested near amount.");
                }

                let investment_withdrawal_info = match self.management_fund.delayed_withdrawn_fund.investment_withdrawal_registry.get(&validator_account_id) {
                    Some(investment_withdrawal_info_) => investment_withdrawal_info_,
                    None => {
                        env::panic_str("Investment withdrawal account is not registered yet.");
                    }
                };
                if near_amount > investment_withdrawal_info.near_amount {
                    env::panic_str("Near amount is more than requested near amount from validator.");
                }
            }
        }

        match validator_info.staking_contract_version {
            ValidatorStakingContractVersion::Classic => {
                validator::ext(validator_account_id.clone())
                    // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                    .unstake(near_amount.into())
                    .then(
                        Self::ext(env::current_account_id())
                            .requested_decrease_validator_stake_callback(
                                validator_account_id,
                                near_amount,
                                stake_decreasing_type,
                                Self::calculate_storage_staking_price(self.management_fund.delayed_withdrawn_fund.storage_usage_per_investment_withdrawal)
                            )
                    )
            }
        }
    }

    fn internal_take_unstaked_balance(&mut self, validator_account_id: AccountId) -> Promise {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_desynchronized();
        self.assert_authorized_management_only_by_manager();

        let current_epoch_height = env::epoch_height();

        if !Self::epoch_is_right(current_epoch_height) {
            env::panic_str("Epoch is not intended for a take unstaked balance.");
        }

        match self.validating_node.validator_registry.get(&validator_account_id) {   // TODO // TODO ЧТо будет, если валидатор перестал работать, что придет с контракта. Не прервется ли из-за этго цепочка выполнения апдейтов
            Some(validator_info) => {
                if validator_info.unstaked_balance == 0 {
                    env::panic_str("Insufficient unstaked balance on validator.");
                }
                if validator_info.last_update_info_epoch_height >= current_epoch_height {       // TODO нужно ли это услвоие
                    env::panic_str("Validator is already updated.");
                }

                match validator_info.staking_contract_version {
                    ValidatorStakingContractVersion::Classic => {
                        validator::ext(validator_account_id.clone())
                            // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                            .withdraw(validator_info.unstaked_balance.into())
                            .then(
                                Self::ext(env::current_account_id())
                                    .take_unstaked_balance_callback(validator_account_id)
                            )
                    }
                }
            }
            None => {
                env::panic_str("Validator account is not registered yet.");
            }
        }
    }

    fn internal_update_validator_info(&mut self, validator_account_id: AccountId) -> Promise {     // TODO TODO TODO Что делать, если в новой эпохе часть обновилась, и уже еще раз наступила новая эпоха, и теперь то, что осталось, обновились. То есть, рассинхронизация состояния.   // TODO Сюда нужно зафиксировать максимальное число Газа. Возможно ли?
        Self::assert_gas_is_enough();
        self.assert_epoch_is_desynchronized();
        self.assert_authorized_management_only_by_manager();

        match self.validating_node.validator_registry.get(&validator_account_id) {   // TODO // TODO ЧТо будет, если валидатор перестал работать, что придет с контракта. Не прервется ли из-за этго цепочка выполнения апдейтов
            Some(validator_info) => {
                let current_epoch_height = env::epoch_height();

                if validator_info.last_update_info_epoch_height < current_epoch_height {
                    match validator_info.staking_contract_version {
                        ValidatorStakingContractVersion::Classic => {
                            return validator::ext(validator_account_id.clone())
                                // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                                .get_account_staked_balance(env::current_account_id())
                                .then(
                                    Self::ext(env::current_account_id())
                                        .update_validator_info_callback(validator_account_id, current_epoch_height)
                                )
                        }
                    }
                }

                env::panic_str("Validator is already updated.");
            }
            None => {
                env::panic_str("Validator account is not registered yet.");
            }
        }
    }

    fn internal_update(&mut self) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_desynchronized();
        self.assert_authorized_management_only_by_manager();

        if self.validating_node.quantity_of_validators_updated_in_current_epoch != self.validating_node.validators_quantity {
            env::panic_str("Some validators are not updated.");
        }

        let current_epoch_height = env::epoch_height();

        if Self::epoch_is_right(current_epoch_height)
            && (self.management_fund.delayed_withdrawn_fund.needed_to_request_classic_near_amount > 0
                || self.management_fund.delayed_withdrawn_fund.needed_to_request_investment_near_amount > 0) {
                env::panic_str("Some funds are not unstaked from validators.");
        }

        let previous_epoch_rewards_from_validators_token_amount = self.convert_near_amount_to_token_amount(
            self.previous_epoch_rewards_from_validators_near_amount
        );

        self.management_fund.staked_balance += self.previous_epoch_rewards_from_validators_near_amount;
        self.management_fund.is_distributed_on_validators_in_current_epoch = false;
        self.validating_node.quantity_of_validators_updated_in_current_epoch = 0;
        self.current_epoch_height = current_epoch_height;
        self.total_rewards_from_validators_near_amount += self.previous_epoch_rewards_from_validators_near_amount;
        self.previous_epoch_rewards_from_validators_near_amount = 0;                               // TODO переназвать, Убрать в впомагательные параметры.

        if let Some(ref rewards_fee) = self.fee_registry.rewards_fee {
            let rewards_fee_token_amount = rewards_fee.multiply(previous_epoch_rewards_from_validators_token_amount);
            if rewards_fee_token_amount != 0 {
                match self.fungible_token.account_registry.get(&self.rewards_receiver_account_id) {
                    Some(mut token_balance) => {
                        token_balance += rewards_fee_token_amount;

                        self.fungible_token.total_supply += rewards_fee_token_amount;
                        self.fungible_token.account_registry.insert(&self.rewards_receiver_account_id, &token_balance);
                    }
                    None => {
                        env::panic_str("Object should exist.");
                    }
                }
            }

            if let Some(ref everstake_rewards_fee) = self.fee_registry.everstake_rewards_fee {
                let everstake_rewards_fee_token_amount = everstake_rewards_fee.multiply(rewards_fee_token_amount);
                if everstake_rewards_fee_token_amount != 0 {
                    match self.fungible_token.account_registry.get(&self.everstake_rewards_receiver_account_id) {
                        Some(mut token_balance) => {
                            token_balance += everstake_rewards_fee_token_amount;

                            self.fungible_token.total_supply += everstake_rewards_fee_token_amount;
                            self.fungible_token.account_registry.insert(&self.everstake_rewards_receiver_account_id, &token_balance);
                        }
                        None => {
                            env::panic_str("Object should exist.");
                        }
                    }
                }
            }
        }
    }

    fn internal_add_validator(
        &mut self,
        validator_account_id: AccountId,
        validator_staking_contract_version: ValidatorStakingContractVersion,
        is_only_for_investment: bool,
        is_preferred: bool
    ) -> PromiseOrValue<()> {                                                     // TODO можно ли проверить, что адрес валиден, и валидатор в вайт-листе?
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        let storage_staking_price_per_additional_validator_account = Self::calculate_storage_staking_price(self.validating_node.storage_usage_per_validator);
        if env::attached_deposit() < storage_staking_price_per_additional_validator_account {
            env::panic_str("Insufficient near deposit.");
        }

        if let Some(maximium_quantity) = self.validating_node.validators_maximum_quantity {
            if self.validating_node.validators_quantity == maximium_quantity {
                env::panic_str("Validator maximum quantity is exceeded.");
            }
        }

        if let Some(_) = self.validating_node.validator_registry.insert(
            &validator_account_id, &ValidatorInfo::new(validator_staking_contract_version, is_only_for_investment)
        ) {
            env::panic_str("Validator account is already registered.");
        }
        self.validating_node.validators_quantity += 1;

        if is_preferred {
            self.validating_node.preffered_validtor = Some(validator_account_id);
        }

        let near_amount = env::attached_deposit() - storage_staking_price_per_additional_validator_account;
        if near_amount > 0 {
            return PromiseOrValue::Promise(
                Promise::new(env::predecessor_account_id())
                    .transfer(near_amount)                                                                  // TODO Нужен ли коллбек?
            );
        }

        PromiseOrValue::Value(())
    }

    fn internal_remove_validator(&mut self, validator_account_id: AccountId) -> Promise {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        match self.validating_node.validator_registry.remove(&validator_account_id) {
            Some(validator_info) => {
                if validator_info.classic_staked_balance > 0
                    || validator_info.investment_staked_balance > 0
                    || validator_info.unstaked_balance > 0 {       // TODO  TODO TODO TODO TODO подумать, при каких условиях еще невозможно удалить валидатор.
                    env::panic_str("Validator has an available balance.");
                }
            }
            None => {
                env::panic_str("Validator account is not registered yet.");
            }
        }

        self.validating_node.validators_quantity -= 1;

        if let Some(ref preffered_validator_account_id) = self.validating_node.preffered_validtor {
            if *preffered_validator_account_id == validator_account_id {
                self.validating_node.preffered_validtor = None;
            }
        }

        let near_amount = Self::calculate_storage_staking_price(self.validating_node.storage_usage_per_validator);

        Promise::new(env::predecessor_account_id())
            .transfer(near_amount)
    }

    fn internal_change_validator_investment_context(&mut self, validator_account_id: AccountId, is_only_for_investment: bool) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        let mut validator_info = match self.validating_node.validator_registry.get(&validator_account_id) {
            Some(validator_info_) => validator_info_,
            None => {
                env::panic_str("Validator account is not registered yet.");
            }
        };

        if validator_info.is_only_for_investment == is_only_for_investment {
            env::panic_str("Changing the state to the same state.");
        }

        validator_info.is_only_for_investment = is_only_for_investment;
        self.validating_node.validator_registry.insert(&validator_account_id, &validator_info);
    }

    fn internal_change_preffered_validator(&mut self, validator_account_id: Option<AccountId>) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        match validator_account_id {
            Some(validator_account_id_) => {
                match self.validating_node.validator_registry.get(&validator_account_id_) {
                    Some(_) => {
                        self.validating_node.preffered_validtor = Some(validator_account_id_);
                    }
                    None => {
                        env::panic_str("Validator account is not registered yet.");
                    }
                }
            }
            None => {
                self.validating_node.preffered_validtor = None;
            }
        }
    }

    fn internal_add_investor(&mut self, investor_account_id: AccountId) -> PromiseOrValue<()> {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        let storage_staking_price_per_additional_investor_account = Self::calculate_storage_staking_price(self.validating_node.storage_usage_per_investor);
        if env::attached_deposit() < storage_staking_price_per_additional_investor_account {
            env::panic_str("Insufficient near deposit.");
        }

        if let Some(_) = self.validating_node.investor_investment_registry.insert(
            &investor_account_id, &InvestorInvestmentInfo::new(investor_account_id.clone())
        ) {
            env::panic_str("Investor account is already registered.");
        }

        let near_amount = env::attached_deposit() - storage_staking_price_per_additional_investor_account;
        if near_amount > 0 {
            return PromiseOrValue::Promise(
                Promise::new(env::predecessor_account_id())
                    .transfer(near_amount)
            );
        }

        PromiseOrValue::Value(())
    }

    fn internal_remove_investor(&mut self, investor_account_id: AccountId) -> Promise {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        let investor_investment_info = match self.validating_node.investor_investment_registry.remove(&investor_account_id) {  // TODO TODO TODO TODO TODO Обратить внимание, что на постоянное количество инвестмент токенов приходится увеличивающееся количество неара, но инвестмент баланс зафиксирован.
            Some(investor_investment_info_) => investor_investment_info_,
            None => {
                env::panic_str("Investor account is not registered yet.");
            }
        };
        if investor_investment_info.staked_balance > 0 || investor_investment_info.distributions_quantity > 0 {
            env::panic_str("Validator has an available balance.");
        }

        let near_amount = Self::calculate_storage_staking_price(self.validating_node.storage_usage_per_investor);

        Promise::new(env::predecessor_account_id())
            .transfer(near_amount)
    }

    fn internal_change_manager(&mut self, manager_id: AccountId) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management();

        self.manager_id = manager_id;
    }

    fn internal_change_rewards_fee(&mut self, rewards_fee: Option<Fee>) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        if let Some(ref rewards_fee_) = rewards_fee {
            rewards_fee_.assert_valid();
        }

        self.fee_registry.rewards_fee = rewards_fee;
    }

    fn internal_change_everstake_rewards_fee(&mut self, everstake_rewards_fee: Option<Fee>) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        if let Some(ref everstake_rewards_fee_) = everstake_rewards_fee {
            everstake_rewards_fee_.assert_valid();
        }

        self.fee_registry.everstake_rewards_fee = everstake_rewards_fee;
    }

    fn internal_confirm_stake_distribution(&mut self) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        self.management_fund.is_distributed_on_validators_in_current_epoch = true;
    }

    fn internal_ft_transfer(&mut self, receiver_account_id: AccountId, token_amount: Balance) -> Promise {     // TODO стоит ли удалять токен-аккаунт с нулевым балансом. Сейчас я его удаляю. нет механизма регистрации. Сейчас регистрируется тоько при депозите
        Self::assert_gas_is_enough();

        if token_amount == 0 {
            env::panic_str("Insufficient token amount.");
        }

        let mut near_amount = env::attached_deposit();
        if near_amount != MINIMUM_ATTACHED_DEPOSIT {
            env::panic_str("Wrong attached deposit.");
        }

        let predecessor_account_id = env::predecessor_account_id();
        let mut predecessor_account_token_balance = match self.fungible_token.account_registry.get(&predecessor_account_id) {
            Some(token_balance) => token_balance,
            None => {
                env::panic_str("Token account is not registered yet.");
            }
        };

        let mut receiver_account_token_balance = match self.fungible_token.account_registry.get(&receiver_account_id) {
            Some(token_balance) => token_balance,
            None => {
                env::panic_str("Token account is not registered yet.");
            }
        };

        if predecessor_account_token_balance < token_amount {
            env::panic_str("Token amount exceeded the available token balance.");
        }

        predecessor_account_token_balance -= token_amount;

        if let Some(investor_investment_info) = self.validating_node.investor_investment_registry.get(&predecessor_account_id) {
            if self.convert_token_amount_to_near_amount(predecessor_account_token_balance) < investor_investment_info.staked_balance {
                env::panic_str("Token amount exceeded the available to transfer token amount.");
            }
        }

        receiver_account_token_balance += token_amount;

        if predecessor_account_token_balance > 0
            || predecessor_account_id == self.rewards_receiver_account_id
            || predecessor_account_id == self.everstake_rewards_receiver_account_id {                                           // TODO стоит ли записать здесь и везде два последних условия через метод
            self.fungible_token.account_registry.insert(&predecessor_account_id, &predecessor_account_token_balance);
        } else {
            self.fungible_token.account_registry.remove(&predecessor_account_id);
            self.fungible_token.accounts_quantity -= 1;

            near_amount += Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_account);
        }
        self.fungible_token.account_registry.insert(&receiver_account_id, &receiver_account_token_balance);

        Promise::new(predecessor_account_id)
            .transfer(near_amount)
    }

    pub fn internal_has_delayed_withdrawal(&self, account_id: AccountId) -> bool {
        self.assert_epoch_is_synchronized();

        self.management_fund.delayed_withdrawn_fund.account_registry.contains_key(&account_id)
    }

    pub fn internal_get_delayed_withdrawal_details(&self, account_id: AccountId) -> (u64, U128) {
        self.assert_epoch_is_synchronized();

        match self.management_fund.delayed_withdrawn_fund.account_registry.get(&account_id) {
            Some(delayed_withdrawal_info) =>
                (delayed_withdrawal_info.get_epoch_quantity_to_take_delayed_withdrawal(self.current_epoch_height), delayed_withdrawal_info.near_amount.into()),
            None => {
                env::panic_str("Delayed withdrawal account is not registered yet.");
            }
        }
    }

    fn internal_is_account_registered(&self, account_id: AccountId) -> bool {
        self.fungible_token.account_registry.contains_key(&account_id)
    }

    fn internal_get_total_token_supply(&self) -> Balance {
        self.assert_epoch_is_synchronized();

        self.fungible_token.total_supply
    }

    fn internal_get_stakers_quantity(&self) -> u64 {
        self.fungible_token.accounts_quantity
    }

    pub fn internal_get_storage_staking_price(&self) -> StorageStakingPrice {
        StorageStakingPrice {
            per_delayed_withdrawal_fund_account: Self::calculate_storage_staking_price(self.management_fund.delayed_withdrawn_fund.storage_usage_per_account).into(),
            per_delayed_withdrawal_fund_investment_withdrawal: Self::calculate_storage_staking_price(self.management_fund.delayed_withdrawn_fund.storage_usage_per_investment_withdrawal).into(),
            per_fungible_token_account: Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_account).into(),
            per_validating_node_validator: Self::calculate_storage_staking_price(self.validating_node.storage_usage_per_validator).into(),
            per_validating_node_investor: Self::calculate_storage_staking_price(self.validating_node.storage_usage_per_investor).into(),
            per_validating_node_distribution: Self::calculate_storage_staking_price(self.validating_node.storage_usage_per_distribution).into()
        }
    }

    fn internal_get_account_balance(&self, account_id: AccountId) -> (U128, U128, U128, U128) {
        let token_balance = match self.fungible_token.account_registry.get(&account_id) {
            Some(token_balance_) => token_balance_,
            None => {
                env::panic_str("Token account is not registered yet.");
            }
        };

        let common_near_balance = self.convert_token_amount_to_near_amount(token_balance);

        let investment_near_balance = match self.validating_node.investor_investment_registry.get(&account_id) {
            Some(investor_investment_info_) => investor_investment_info_.staked_balance,
            None => 0
        };

        if common_near_balance < investment_near_balance {
            env::panic_str("Nonexecutable code. Near balance should be greater then or equal to investment near balance.");
        }

        (token_balance.into(), common_near_balance.into(), (common_near_balance - investment_near_balance).into(), investment_near_balance.into())
    }

    fn internal_get_management_fund(&self) -> (U128, U128, U128) {
        self.assert_epoch_is_synchronized();

        (
            self.management_fund.unstaked_balance.into(),
            self.management_fund.staked_balance.into(),
            self.management_fund.get_management_fund_amount().into()
        )
    }

    fn internal_get_fee_registry(&self) -> FeeRegistry {
        self.assert_epoch_is_synchronized();

        self.fee_registry.clone()
    }

    pub fn internal_get_current_epoch_height(&self) -> (EpochHeight, EpochHeight) {
        (self.current_epoch_height, env::epoch_height())
    }

    pub fn internal_is_stake_distributed(&self) -> bool {
        self.management_fund.is_distributed_on_validators_in_current_epoch
    }

    pub fn internal_is_investor(&self, account_id: AccountId) -> bool {
        self.assert_epoch_is_synchronized();

        self.validating_node.investor_investment_registry.contains_key(&account_id)
    }

    pub fn internal_get_investor_investment_info(&self, account_id: AccountId) -> InvestorInvestmentInfoDto {
        self.assert_epoch_is_synchronized();

        let mut distribution_registry: Vec<(AccountId, U128)> = vec![];

        let investor_investment_info = match self.validating_node.investor_investment_registry.get(&account_id) {
            Some(investor_investment_info_) => investor_investment_info_,
            None => {
                env::panic_str("Investor account is not registered yet.");
            }
        };

        for validator_account_id in self.validating_node.validator_registry.keys() {
            if let Some(staked_balance) = investor_investment_info.distribution_registry.get(&validator_account_id) {
                distribution_registry.push((validator_account_id, staked_balance.into()));
            }
        }

        InvestorInvestmentInfoDto {
            distribution_registry,
            staked_balance: investor_investment_info.staked_balance.into()
        }
    }

    fn internal_get_validator_info_registry(&self) -> Vec<ValidatorInfoDto> {
        let mut validator_info_dto_registry: Vec<ValidatorInfoDto> = vec![];

        for (account_id, validator_info) in self.validating_node.validator_registry.into_iter() {
            let ValidatorInfo {
                classic_staked_balance,
                investment_staked_balance,
                unstaked_balance,
                staking_contract_version: _,
                is_only_for_investment,
                last_update_info_epoch_height,
                last_classic_stake_increasing_epoch_height
            } = validator_info;

            validator_info_dto_registry.push(
                ValidatorInfoDto {
                    account_id,
                    classic_staked_balance: classic_staked_balance.into(),
                    investment_staked_balance: investment_staked_balance.into(),
                    unstaked_balance: unstaked_balance.into(),
                    is_only_for_investment,
                    last_update_info_epoch_height,
                    last_stake_increasing_epoch_height: last_classic_stake_increasing_epoch_height
                }
            );
        }

        validator_info_dto_registry
    }

    fn internal_get_aggregated_info(&self) -> AggregatedInfo {
        self.assert_epoch_is_synchronized();

        AggregatedInfo {
            unstaked_balance: self.management_fund.unstaked_balance.into(),
            staked_balance: self.management_fund.staked_balance.into(),
            token_total_supply: self.fungible_token.total_supply.into(),
            token_accounts_quantity: self.fungible_token.accounts_quantity,
            total_rewards_from_validators_near_amount: self.total_rewards_from_validators_near_amount.into(),
            rewards_fee: self.fee_registry.rewards_fee.clone()
        }
    }

    fn internal_get_requested_to_withdrawal_fund(&self) -> RequestedToWithdrawalFund {
        let mut investment_withdrawal_registry: Vec<(AccountId, U128)> = vec![];

        for validator_account_id in self.validating_node.validator_registry.keys() {
            if let Some(investment_withdrawal_info) = self.management_fund.delayed_withdrawn_fund.investment_withdrawal_registry.get(&validator_account_id) {
                investment_withdrawal_registry.push((validator_account_id, investment_withdrawal_info.near_amount.into()))
            }
        }

        RequestedToWithdrawalFund {
            classic_near_amount: self.management_fund.delayed_withdrawn_fund.needed_to_request_classic_near_amount.into(),
            investment_near_amount: self.management_fund.delayed_withdrawn_fund.needed_to_request_investment_near_amount.into(),
            investment_withdrawal_registry
        }
    }

    fn internal_ft_total_supply(&self) -> Balance {
        self.fungible_token.total_supply
    }

    fn internal_ft_balance_of(&self, account_id: AccountId) -> Balance {
        match self.fungible_token.account_registry.get(&account_id) {
            Some(token_balance) => token_balance,
            None => 0
        }
    }

    fn convert_near_amount_to_token_amount(&self, near_amount: Balance) -> Balance {
        if self.management_fund.get_management_fund_amount() == 0 {
            return near_amount;
        }

        (
            U256::from(near_amount)
            * U256::from(self.fungible_token.total_supply)
            / U256::from(self.management_fund.get_management_fund_amount())             // TODO Проверить Округление
        ).as_u128()
    }

    fn convert_token_amount_to_near_amount(&self, token_amount: Balance) -> Balance {      // TOD вот здесь обратить внимание. Правильно ли стоит проверка в случае, если здесь ноль, а неаров не ноль. ТАкое может быть в контексте получения и вывда ревардов
        if self.fungible_token.total_supply == 0 {
            return token_amount
        }

        (
            U256::from(token_amount)
            * U256::from(self.management_fund.get_management_fund_amount())             // TODO Проверить Округление
            / U256::from(self.fungible_token.total_supply)
        ).as_u128()
    }

    fn assert_authorized_management_only_by_manager(&self) {
        if self.manager_id != env::predecessor_account_id() {
            env::panic_str("Unauthorized management. Management must be carried out either by the manager of the pool.");
        }
    }

    fn assert_authorized_management(&self) {
        let predecessor_account_id = env::predecessor_account_id();

        if self.owner_id != predecessor_account_id && self.manager_id != predecessor_account_id {
            env::panic_str("Unauthorized management. Management must be carried out either by the owner or manager of the pool.");
        }
    }

    fn assert_epoch_is_synchronized(&self) {
        if self.current_epoch_height != env::epoch_height() {
            env::panic_str("Epoch should be in synchronized state.");
        }
    }

    fn assert_epoch_is_desynchronized(&self) {
        if self.current_epoch_height == env::epoch_height() {
            env::panic_str("Epoch should be in desynchronized state.");
        }
    }

    fn assert_gas_is_enough() {        // TODO проссчитать Количество Газа для каждого метода и вставить сюда в сигнатуру.
        if env::prepaid_gas() < (Gas::ONE_TERA * MAXIMUM_NUMBER_OF_TGAS) {
            env::panic_str("Not enough Gas quantity.");
        }
    }

    fn epoch_is_right(epoch_height: EpochHeight) -> bool {
        (epoch_height % 4) == 0                                                      // TODO  цифру 4 - в константу положить.
    }

    fn calculate_storage_staking_price(quantity_of_bytes: StorageUsage) -> Balance {
        match Balance::from(quantity_of_bytes).checked_mul(env::storage_byte_cost()) {
            Some(storage_staking_price) => storage_staking_price,
            None => {
                env::panic_str("Calculation overflow.");
            }
        }
    }
}

#[near_bindgen]
impl StakePool {
    #[private]
    pub fn deposit_callback(
        &mut self,
        predecessor_account_id: AccountId,
        validator_account_id: AccountId,
        near_amount: Balance,
        token_amount: Balance,
        current_epoch_height: EpochHeight
    ) {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let mut validator_info = match self.validating_node.validator_registry.get(&validator_account_id) {
                    Some(validator_info_) => validator_info_,
                    None => {
                        env::panic_str("Nonexecutable code. Account must exist.");
                    }
                };
                validator_info.classic_staked_balance += near_amount;
                validator_info.last_classic_stake_increasing_epoch_height = Some(current_epoch_height);
                self.validating_node.validator_registry.insert(&validator_account_id, &validator_info);

                self.management_fund.staked_balance += near_amount;
            }
            _ => {
                self.management_fund.unstaked_balance += near_amount;
            }
        }

        let mut token_balance = match self.fungible_token.account_registry.get(&predecessor_account_id) {
            Some(token_balance_) => token_balance_,
            None => {
                self.fungible_token.accounts_quantity += 1;

                0
            }
        };
        token_balance += token_amount;
        self.fungible_token.account_registry.insert(&predecessor_account_id, &token_balance);
        self.fungible_token.total_supply += token_amount;
    }

    #[private]
    pub fn deposit_on_validator_callback(
        &mut self,
        predecessor_account_id: AccountId,
        validator_account_id: AccountId,
        near_amount: Balance,
        attached_deposit: Balance,
        refundable_near_amount: Balance,
        token_amount: Balance
    ) {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let mut validator_info = match self.validating_node.validator_registry.get(&validator_account_id) {
                    Some(validator_info_) => validator_info_,
                    None => {
                        env::panic_str("Nonexecutable code. Account must exist.");
                    }
                };
                validator_info.investment_staked_balance += near_amount;
                self.validating_node.validator_registry.insert(&validator_account_id, &validator_info);

                let mut investor_investment_info = match self.validating_node.investor_investment_registry.get(&predecessor_account_id) {
                    Some(investor_investment_info_) => investor_investment_info_,
                    None => {
                        env::panic_str("Nonexecutable code. Account must exist.");
                    }
                };
                let mut staked_balance = match investor_investment_info.distribution_registry.get(&validator_account_id) {
                    Some(staked_balance_) => staked_balance_,
                    None => {
                        investor_investment_info.distributions_quantity += 1;

                        0
                    }
                };
                staked_balance += near_amount;
                investor_investment_info.distribution_registry.insert(&validator_account_id, &staked_balance);
                investor_investment_info.staked_balance += near_amount;
                self.validating_node.investor_investment_registry.insert(&predecessor_account_id, &investor_investment_info);

                let mut token_balance = match self.fungible_token.account_registry.get(&predecessor_account_id) {
                    Some(token_balance_) => token_balance_,
                    None => {
                        self.fungible_token.accounts_quantity += 1;

                        0
                    }
                };
                token_balance += token_amount;
                self.fungible_token.account_registry.insert(&predecessor_account_id, &token_balance);
                self.fungible_token.total_supply += token_amount;

                self.management_fund.staked_balance += near_amount;

                if refundable_near_amount > 0 {
                    Promise::new(predecessor_account_id)
                        .transfer(refundable_near_amount);
                }
            }
            _ => {
                Promise::new(predecessor_account_id)
                    .transfer(attached_deposit);
            }
        }
    }

    #[private]
    pub fn increase_validator_stake_callback(
        &mut self,
        validator_account_id: AccountId,
        near_amount: Balance,
        current_epoch_height: EpochHeight
    ) -> bool {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");        // TODO Фраза повторяется. Нужно ли выновсить в константу?
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                self.management_fund.unstaked_balance -= near_amount;
                self.management_fund.staked_balance += near_amount;

                let mut validator_info = match self.validating_node.validator_registry.get(&validator_account_id) {
                    Some(validator_info_) => validator_info_,
                    None => {
                        env::panic_str("Nonexecutable code. Account must exist.");
                    }
                };
                validator_info.classic_staked_balance += near_amount;
                validator_info.last_classic_stake_increasing_epoch_height = Some(current_epoch_height);
                self.validating_node.validator_registry.insert(&validator_account_id, &validator_info);

                true
            }
            _ => {
                false
            }
        }
    }

    #[private]
    pub fn requested_decrease_validator_stake_callback(    // TODO TODO TODO Это дикриз для нужнд пользоватпеля, но подойдет ли этот метод, если мы хотим просто сделать дикриз стейка валидатора, с целью перераспределения. Обратить внимание на то, что то, что в ДелайдВитхдровол уже не влияет на курс. TODO TODO TODO написать метод, который снимает для нужд менеджера, при этом дать возможность пользователяи продолжать делать ДелайдАнстейк, даже если мы для нужнд менеджера запросили все средства.!!!!!!!!    // TODO TODO проверить, что во всех методах, где есть коллбек, нет изменения состояния вне коллбека
        &mut self,
        validator_account_id: AccountId,
        near_amount: Balance,
        stake_decreasing_type: StakeDecreasingType,
        refundable_near_amount: Balance
    ) -> (bool, EpochHeight) {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let mut validator_info = match self.validating_node.validator_registry.get(&validator_account_id) {
                    Some(validator_info_) => validator_info_,
                    None => {
                        env::panic_str("Nonexecutable code. Account must exist.");
                    }
                };

                match stake_decreasing_type {
                    StakeDecreasingType::Classic => {
                        validator_info.classic_staked_balance -= near_amount;
                        self.management_fund.delayed_withdrawn_fund.needed_to_request_classic_near_amount -= near_amount;
                    }
                    StakeDecreasingType::Investment => {
                        let mut investment_withdrawal_info = match self.management_fund.delayed_withdrawn_fund.investment_withdrawal_registry.get(&validator_account_id) {
                            Some(investment_withdrawal_info_) => investment_withdrawal_info_,
                            None => {
                                env::panic_str("Nonexecutable code. Account must exist.");
                            }
                        };
                        if near_amount < investment_withdrawal_info.near_amount {
                            investment_withdrawal_info.near_amount -= near_amount;

                            self.management_fund.delayed_withdrawn_fund.investment_withdrawal_registry.insert(&validator_account_id, &investment_withdrawal_info);
                        } else {
                            self.management_fund.delayed_withdrawn_fund.investment_withdrawal_registry.remove(&validator_account_id);

                            Promise::new(investment_withdrawal_info.account_id)
                                .transfer(refundable_near_amount);
                        }

                        validator_info.investment_staked_balance -= near_amount;
                        self.management_fund.delayed_withdrawn_fund.needed_to_request_investment_near_amount -= near_amount;
                    }
                }

                validator_info.unstaked_balance += near_amount;
                self.validating_node.validator_registry.insert(&validator_account_id, &validator_info);

                (true, env::epoch_height())
            }
            _ => {
                (false, env::epoch_height())
            }
        }
    }

    #[private]
    pub fn take_unstaked_balance_callback(&mut self, validator_account_id: AccountId) -> (bool, EpochHeight) {  // TODO Может быть, ставить счетчик на количество валиаторов, с которыз нужно снимать стейк, чтобы проверять.
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let mut validator_info = match self.validating_node.validator_registry.get(&validator_account_id) {
                    Some(validator_info_) => validator_info_,
                    None => {
                        env::panic_str("Nonexecutable code. Account must exist.");
                    }
                };

                self.management_fund.delayed_withdrawn_fund.balance += validator_info.unstaked_balance;

                validator_info.unstaked_balance = 0;
                self.validating_node.validator_registry.insert(&validator_account_id, &validator_info);

                (true, env::epoch_height())
            }
            _ => {
                (false, env::epoch_height())
            }
        }
    }

    // TODO комментарий написать. Возвращаем и сохраняем епохи в разном состоянии по-разному, чтобы запомнить, что в какой эпохе инициировано по фактту, а в какую выполнен коллбек
    #[private]
    pub fn update_validator_info_callback(
        &mut self,
        validator_account_id: AccountId,
        current_epoch_height: EpochHeight
    ) -> (bool, EpochHeight) {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(data) => {
                let new_staked_balance: u128 = match near_sdk::serde_json::from_slice::<U128>(data.as_slice()) {  // TODO Проверить, правилен ли тот факт, что нужно обработать Джсон в ДжсонТипРаст
                    Ok(new_staked_balance_) => new_staked_balance_.into(),
                    Err(_) => {
                        env::panic_str("Nonexecutable code. It should be valid JSON object.");
                    }
                };

                let mut validator_info = match self.validating_node.validator_registry.get(&validator_account_id) {
                    Some(validator_info_) => validator_info_,
                    None => {
                        env::panic_str("Nonexecutable code. Account must exist.");
                    }
                };

                let current_staked_balance = validator_info.classic_staked_balance + validator_info.investment_staked_balance;

                let staking_rewards_near_amount = new_staked_balance - current_staked_balance;

                validator_info.last_update_info_epoch_height = current_epoch_height;
                validator_info.classic_staked_balance = new_staked_balance - validator_info.investment_staked_balance;

                self.validating_node.validator_registry.insert(&validator_account_id, &validator_info);
                self.validating_node.quantity_of_validators_updated_in_current_epoch += 1;

                self.previous_epoch_rewards_from_validators_near_amount += staking_rewards_near_amount;

                (true, env::epoch_height())
            }
            _ => {
                (false, env::epoch_height())
            }
        }
    }
}

// TODO Понять работу с аккаунтами. КОму пренадлжат, кто может мзенять состояние, и подобные вещи

// #[ext_contract(ext_voting)]           что это такое???????????????????????????????????????????????
// pub trait VoteContract {
//     /// Method for validators to vote or withdraw the vote.
//     /// Votes for if `is_vote` is true, or withdraws the vote if `is_vote` is false.
//     fn vote(&mut self, is_vote: bool);
// }


//#[global_allocator]
// static ALLOC: near_sdk::wee_alloc::WeeAlloc = near_sdk::wee_alloc::WeeAlloc::INIT;            Нужно ли вот это ??????????????????
// near_sdk::setup_alloc!();


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


//  TODO TODO TODO ВСе Итераторы на Вью делать через индекс.     https://github.com/NearDeFi/burrowland/blob/0dbfa1803bf26353ffbee2ffd4f494bab23b2756/contract/src/account.rs#L207

// TODO TODO TODO TODO TODO Важно запрашивать необходимое количество газа, чтобы хватило на  контракт + кроссколл + коллбек. Иначе что-то выполнится, а что-то нет.

// TODO C Валидатора, по идее, придет немного больше неар, чем запрошено по методам, так как мы ожидаем запрос на Анстейк 4 эпохи, в это время количество отдаваемого зафиксировано, но оно еще приносит прибыль (затем еще 4 эпохи, чтобы забрать), что с этим делать?

// TODO В каждом методе проверить, что все, что взято из хранилища, в него и положено. (get, insert.) .
// TODO проверить, что взятые из хранилища параметры изменяются там, где это требуется.

// TODO проверить, нет ли такого, чтобы пользователб мог что-то сделать за другого пользователя. То есть. АккаунтАйди передается в сигнатуру, а не берется ПредецессорАккаунтАйди

// TODO, пороверить все ли методы нужны.

// TODO TODO TODO TODO TODO Можно ли будет перейти на МУЛЬТИСИГ флоу управления после деплоя классического флоу управления.

// TODO логировать

// проверить, что у каждого свойства структуры есть инкремент и дикремент.

// написать документацию.

// Убрать ли везде Инфо?


// https://nomicon.io/Standards  Евенты и подобное.



// // use near_contract_standards::fungible_token::core::FungibleTokenCore;
// use near_contract_standards::fungible_token::events  - посмотреть, какие нужны
// use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
// https://learn.figment.io/tutorials/stake-fungible-token

// Нужна ли фабрика?

// TODO проработать логи, то есть, втсавлять в них конкретные значения






// НАписать DecreaseValidatorStake относительно менеджера, причем это не должно влиять на возможность снятия средств пользователями.