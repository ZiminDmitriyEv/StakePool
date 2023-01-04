use core::convert::Into;
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_sdk::{env, near_bindgen, PanicOnDefault, AccountId, Balance, EpochHeight, Promise, PromiseResult, StorageUsage, Gas, PromiseOrValue};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use super::account_registry::AccountRegistry;
use super::cross_contract_call::validator::validator;
use super::data_transfer_object::account_balance::AccountBalance;
use super::data_transfer_object::aggregated::Aggregated;
use super::data_transfer_object::base_account_balance::BaseAccountBalance;
use super::data_transfer_object::callback_result::CallbackResult;
use super::data_transfer_object::delayed_withdrawal_details::DelayedWithdrawalDetails;
use super::data_transfer_object::epoch_height_registry::EpochHeightRegistry;
use super::data_transfer_object::fee_registry_light::FeeRegistryLight;
use super::data_transfer_object::full_for_account::FullForAccount;
use super::data_transfer_object::full::Full;
use super::data_transfer_object::fund::Fund as FundDto;
use super::data_transfer_object::investor_investment::InvestorInvestment as InvestorInvestmentDto;
use super::data_transfer_object::requested_to_withdrawal_fund::RequestedToWithdrawalFund;
use super::data_transfer_object::storage_staking_price::StorageStakingPrice;
use super::data_transfer_object::storage_staking_requested_coverage::StorageStakingRequestedCoverage;
use super::data_transfer_object::validator::Validator as ValidatorDto;
use super::delayed_withdrawal::DelayedWithdrawal;
use super::fee_registry::FeeRegistry;
use super::fee::Fee;
use super::fund::Fund;
use super::fungible_token::FungibleToken;
use super::investment_withdrawal::InvestmentWithdrawal;
use super::investor_investment::InvestorInvestment;
use super::MAXIMUM_NUMBER_OF_TGAS;
use super::MINIMUN_DEPOSIT_AMOUNT;
use super::shared_fee::SharedFee;
use super::stake_decreasing_kind::StakeDecreasingType;
use super::staking_contract_version::StakingContractVersion;
use super::validating::Validating;
use super::validator::Validator;
use uint::construct_uint;

construct_uint! {
    pub struct U256(4);
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]     // TODO проверить все типы данных. LazyOption, например, добавить там, где Мэпы и сеты, посмотреть, где нужно !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!1
pub struct StakePool {
    account_registry: AccountRegistry,
    fungible_token: FungibleToken,
    fund: Fund,
    fee_registry: FeeRegistry,                          // TODO сделать через Next epoch.
    validating: Validating,             // TODO как назвать это поле.
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
        self_fee_receiver_account_id: AccountId,
        partner_fee_receiver_account_id: AccountId,
        reward_fee_self: Option<Fee>,
        reward_fee_partner: Option<Fee>,
        instant_withdraw_fee_self: Option<Fee>,
        instant_withdraw_fee_partner: Option<Fee>
    ) -> Self {      // TODO Сюда заходит дипозит. Как его отсечь, то есть, взять ту часть, к
        Self::internal_new(
            manager_id,
            self_fee_receiver_account_id,
            partner_fee_receiver_account_id,
            reward_fee_self,
            reward_fee_partner,
            instant_withdraw_fee_self,
            instant_withdraw_fee_partner
        )
    }

    /// Stake process.
    #[payable]
    pub fn deposit(&mut self, near_amount: U128) -> PromiseOrValue<()> {
        self.internal_deposit(near_amount.into())
    }

    /// Stake process directly to the validator.
    /// Available only for Investor.
    #[payable]
    pub fn deposit_on_validator(&mut self, near_amount: U128, validator_account_id: AccountId) -> Promise {
        self.internal_deposit_on_validator(near_amount.into(), validator_account_id)
    }

    /// Instant unstake process.
    #[payable]
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

    #[payable]
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

    pub fn update_validator(&mut self, validator_account_id: AccountId) -> Promise {
        self.internal_update_validator(validator_account_id)
    }

    pub fn update(&mut self) {
        self.internal_update();
    }

    #[payable]
    pub fn add_validator(
        &mut self,
        validator_account_id: AccountId,
        staking_contract_version: StakingContractVersion,
        is_only_for_investment: bool,
        is_preferred: bool
    ) -> PromiseOrValue<()> {
        self.internal_add_validator(validator_account_id, staking_contract_version, is_only_for_investment, is_preferred)
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

    pub fn change_reward_fee(&mut self, reward_fee_self: Option<Fee>, reward_fee_partner: Option<Fee>) {
        self.internal_change_reward_fee(reward_fee_self, reward_fee_partner);
    }

    pub fn change_instant_withdraw_fee(&mut self, instant_withdraw_fee_self: Option<Fee>, instant_withdraw_fee_partner: Option<Fee>) {
        self.internal_change_instant_withdraw_fee(instant_withdraw_fee_self, instant_withdraw_fee_partner);
    }

    pub fn confirm_stake_distribution(&mut self) {
        self.internal_confirm_stake_distribution();
    }

    /// View-methods:

    pub fn get_delayed_withdrawal_details(&self, account_id: AccountId) -> Option<DelayedWithdrawalDetails> {
        self.internal_get_delayed_withdrawal_details(account_id)
    }

    pub fn get_account_balance(&self, account_id: AccountId) -> AccountBalance {
        self.internal_get_account_balance(account_id)
    }

    pub fn get_total_token_supply(&self) -> U128 {
        self.internal_get_total_token_supply().into()
    }

    pub fn get_minimum_deposit_amount(&self) -> U128 {
        self.internal_get_minimum_deposit_amount().into()
    }

    pub fn get_storage_staking_price(&self) -> StorageStakingPrice {
        self.internal_get_storage_staking_price()
    }

    pub fn get_storage_staking_requested_coverage(&self, account_id: AccountId) -> StorageStakingRequestedCoverage {
        self.internal_get_storage_staking_requested_coverage(account_id)
    }

    pub fn get_fund(&self) -> FundDto {
        self.internal_get_fund()
    }

    pub fn get_fee_registry(&self) -> FeeRegistry {
        self.internal_get_fee_registry()
    }

    pub fn get_fee_registry_light(&self) -> FeeRegistryLight {
        self.internal_get_fee_registry_light()
    }

    pub fn get_current_epoch_height(&self) -> EpochHeightRegistry {
        self.internal_get_current_epoch_height()
    }

    pub fn is_stake_distributed(&self) -> bool {
        self.internal_is_stake_distributed()
    }

    pub fn get_investor_investment(&self, account_id: AccountId) -> Option<InvestorInvestmentDto> {
        self.internal_get_investor_investment(account_id)
    }

    pub fn get_validator_registry(&self) -> Vec<ValidatorDto> {
        self.internal_get_validator_registry()
    }

    pub fn get_preffered_validator(&self) -> Option<ValidatorDto> {
        self.internal_get_preffered_validator()
    }

    pub fn get_aggregated(&self) -> Aggregated {
        self.internal_get_aggregated()
    }

    pub fn get_requested_to_withdrawal_fund(&self) -> RequestedToWithdrawalFund {
        self.internal_get_requested_to_withdrawal_fund()
    }

    pub fn get_full(&self) -> Full {
        self.internal_get_full()
    }

    pub fn get_full_for_account(&self, account_id: AccountId) -> FullForAccount {
        self.internal_get_full_for_account(account_id)
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
        self_fee_receiver_account_id: AccountId,
        partner_fee_receiver_account_id: AccountId,
        reward_fee_self: Option<Fee>,
        reward_fee_partner: Option<Fee>,
        instant_withdraw_fee_self: Option<Fee>,
        instant_withdraw_fee_partner: Option<Fee>
    ) -> Self {                                     // TODO Посмотреть, сколько нужно для хранения всего стейта ниже. Остальной депозит пололожить в качестве стейка.
        if env::state_exists() {
            env::panic_str("Contract state is already initialize.");
        }

        if self_fee_receiver_account_id == partner_fee_receiver_account_id {
            env::panic_str("The self fee receiver account and partner fee receiver account can not be the same.");
        }

        if reward_fee_self.is_none() && reward_fee_partner.is_some() {
            env::panic_str("Reward fees are not valid.");
        }
        let reward_fee = if let Some(reward_fee_self_) = reward_fee_self {
            reward_fee_self_.assert_valid();

            if let Some(ref reward_fee_partner_) = reward_fee_partner {
                reward_fee_partner_.assert_valid();
            }

            Some (
                SharedFee {
                    self_fee: reward_fee_self_,
                    partner_fee: reward_fee_partner
                }
            )
        } else {
            None
        };

        if instant_withdraw_fee_self.is_none() && instant_withdraw_fee_partner.is_some() {
            env::panic_str("Instant withdraw fees are not valid.");
        }
        let instant_withdraw_fee = if let Some(instant_withdraw_fee_self_) = instant_withdraw_fee_self {
            instant_withdraw_fee_self_.assert_valid();

            if let Some(ref instant_withdraw_fee_partner_) = instant_withdraw_fee_partner {
                instant_withdraw_fee_partner_.assert_valid();
            }

            Some (
                SharedFee {
                    self_fee: instant_withdraw_fee_self_,
                    partner_fee: instant_withdraw_fee_partner
                }
            )
        } else {
            None
        };

        let predecessor_account_id = env::predecessor_account_id();

        let manager_id_ = match manager_id {
            Some(manager_id__) => manager_id__,
            None => predecessor_account_id.clone()
        };

        let mut stake_pool = Self {
            account_registry: AccountRegistry {
                owner_id: predecessor_account_id.clone(),
                manager_id: manager_id_,
                self_fee_receiver_account_id,
                partner_fee_receiver_account_id
            },
            fee_registry: FeeRegistry {
                reward_fee,
                instant_withdraw_fee
            },
            fungible_token: FungibleToken::new(predecessor_account_id),
            fund: Fund::new(),
            validating: Validating::new(),
            current_epoch_height: env::epoch_height(),
            previous_epoch_rewards_from_validators_near_amount: 0,
            total_rewards_from_validators_near_amount: 0
        };
        stake_pool.fungible_token.account_registry.insert(&stake_pool.account_registry.self_fee_receiver_account_id, &0);
        stake_pool.fungible_token.account_registry.insert(&stake_pool.account_registry.partner_fee_receiver_account_id, &0);
        stake_pool.fungible_token.accounts_quantity = 2;

        stake_pool
    }

    fn internal_deposit(&mut self, near_amount: Balance) -> PromiseOrValue<()> {
        Self::assert_gas_is_enough();
        Self::assert_minimum_deposit();
        self.assert_epoch_is_synchronized();

        let predecessor_account_id = env::predecessor_account_id();

        let attached_deposit = env::attached_deposit();

        let (storage_staking_price_per_additional_account, mut token_balance) = match self.fungible_token.account_registry.get(&predecessor_account_id) {
            Some(token_balance_) => (0, token_balance_),
            None => {
                (Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_account), 0)
            }
        };

        let minimum_near_amount = if MINIMUN_DEPOSIT_AMOUNT > storage_staking_price_per_additional_account {
            MINIMUN_DEPOSIT_AMOUNT - storage_staking_price_per_additional_account
        } else {
            env::panic_str("Logic error.");
        };
        if near_amount < minimum_near_amount {
            env::panic_str("Near amount less then minimum required near amount.");
        }

        if attached_deposit < storage_staking_price_per_additional_account {
            env::panic_str("Insufficient near deposit.");
        }

        let available_for_staking_near_amount = attached_deposit - storage_staking_price_per_additional_account;

        if near_amount > available_for_staking_near_amount {
            env::panic_str("Insufficient near deposit.");
        }

        let refundable_near_amount = available_for_staking_near_amount - near_amount;

        let token_amount = self.convert_near_amount_to_token_amount(near_amount);
        if token_amount == 0 {
            env::panic_str("Insufficient near amount.");
        }

        if self.fund.is_distributed_on_validators_in_current_epoch && self.validating.preffered_validator.is_some() {
            match self.validating.preffered_validator {
                Some(ref preffered_validator_account_id) => {
                    match self.validating.validator_registry.get(preffered_validator_account_id) {
                        Some(validator) => {
                            match validator.staking_contract_version {
                                StakingContractVersion::Classic => {
                                    PromiseOrValue::Promise(
                                        validator::ext(preffered_validator_account_id.clone())
                                            .with_attached_deposit(near_amount)
                                            // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                                            .deposit_and_stake()
                                            .then(
                                                Self::ext(env::current_account_id())
                                                    .deposit_callback(
                                                        predecessor_account_id,
                                                        preffered_validator_account_id.clone(),
                                                        attached_deposit,
                                                        near_amount,
                                                        refundable_near_amount,
                                                        token_amount,
                                                        self.current_epoch_height,
                                                        storage_staking_price_per_additional_account
                                                    )
                                            )
                                    )
                                }
                            }
                        }
                        None => {
                            env::panic_str("Nonexecutable code. Object must exist.");
                        }
                    }
                }
                None => {
                    env::panic_str("Nonexecutable code. Object must exist.");
                }
            }
        } else {
            self.fund.classic_unstaked_balance += near_amount;
            self.fungible_token.total_supply += token_amount;

            token_balance += token_amount;
            if let None = self.fungible_token.account_registry.insert(&predecessor_account_id, &token_balance) {
                self.fungible_token.accounts_quantity += 1;
            }

            if refundable_near_amount > 0 {
                Promise::new(predecessor_account_id.clone())
                    .transfer(refundable_near_amount);
            }

            let current_account_id = env::current_account_id();
            env::log_str(
                format!(
                    "
                    Deposited to @{} in {} epoch.
                    Attached deposit is {} yoctoNear.
                    Exchangeable deposit is {} yoctoNear.
                    Reserved storage staking price is {} yoctoNear.
                    Refundable deposit is {} yoctoNear.
                    Old @{} total supply is {} yoctoStNear.
                    Old @{} balance is {} yoctoNear.
                    Old @{} balance is {} yoctoStNear.
                    @{} received {} yoctoStNear.
                    New @{} balance is {} yoctoStNear.
                    New @{} balance is {} yoctoNear.
                    New @{} total supply is {} yoctoStNear.
                    ",
                    &current_account_id,
                    self.current_epoch_height,
                    attached_deposit,
                    near_amount,
                    storage_staking_price_per_additional_account,
                    refundable_near_amount,
                    &current_account_id,
                    self.fungible_token.total_supply - token_amount,
                    &current_account_id,
                    self.fund.get_common_balance() - near_amount,
                    &predecessor_account_id,
                    token_balance - token_amount,
                    &predecessor_account_id,
                    token_amount,
                    &predecessor_account_id,
                    token_balance,
                    &current_account_id,
                    self.fund.get_common_balance(),
                    &current_account_id,
                    self.fungible_token.total_supply
                ).as_str()
            );

            PromiseOrValue::Value(())
        }
    }

    fn internal_deposit_on_validator(&mut self, near_amount: Balance, validator_account_id: AccountId) -> Promise {
        Self::assert_gas_is_enough();
        Self::assert_minimum_deposit();
        self.assert_epoch_is_synchronized();

        let validator = match self.validating.validator_registry.get(&validator_account_id) {
            Some(validator_) => validator_,
            None => {
                env::panic_str("Validator account is not registered yet.");
            }
        };

        let predecessor_account_id = env::predecessor_account_id();

        let attached_deposit = env::attached_deposit();

        let investor_investment = match self.validating.investor_investment_registry.get(&predecessor_account_id) {
            Some(investor_investment_) => investor_investment_,
            None => {
                env::panic_str("Investor account is not registered yet.");
            }
        };

        let mut storage_staking_price_per_additional_accounts: Balance = 0;

        if let None = investor_investment.distribution_registry.get(&validator_account_id) {
            storage_staking_price_per_additional_accounts += Self::calculate_storage_staking_price(self.validating.storage_usage_per_distribution);
        }

        if let None = self.fungible_token.account_registry.get(&predecessor_account_id) {
            storage_staking_price_per_additional_accounts += Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_account);
        };

        let minimum_near_amount = if MINIMUN_DEPOSIT_AMOUNT > storage_staking_price_per_additional_accounts {
            MINIMUN_DEPOSIT_AMOUNT - storage_staking_price_per_additional_accounts
        } else {
            env::panic_str("Logic error.");
        };
        if near_amount < minimum_near_amount {
            env::panic_str("Near amount less then minimum required near amount.");
        }

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

        match validator.staking_contract_version {
            StakingContractVersion::Classic => {
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
                                token_amount,
                                storage_staking_price_per_additional_accounts
                            )
                    )
            }
        }
    }

    fn internal_instant_withdraw(&mut self, mut token_amount: Balance) -> Promise {
        Self::assert_gas_is_enough();
        Self::assert_natural_deposit();
        self.assert_epoch_is_synchronized();

        let token_amount_ = token_amount;

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

        let token_balance_ = token_balance;

        token_balance -= token_amount;
        if let Some(investor_investment) = self.validating.investor_investment_registry.get(&predecessor_account_id) {
            if self.convert_token_amount_to_near_amount(token_balance) < investor_investment.staked_balance {
                env::panic_str("Token amount exceeded the available to instant withdraw token amount.");
            }
        }

        let mut instant_withdraw_fee_self: Option<Fee> = None;

        if let Some(ref instant_withdraw_fee) = self.fee_registry.instant_withdraw_fee {
            instant_withdraw_fee_self = Some(instant_withdraw_fee.self_fee.clone());

            let mut instant_withdraw_fee_self_token_amount = instant_withdraw_fee.self_fee.multiply(token_amount);
            if instant_withdraw_fee_self_token_amount != 0 {
                token_amount -= instant_withdraw_fee_self_token_amount;

                if let Some(ref instant_withdraw_fee_partner) = instant_withdraw_fee.partner_fee {
                    let instant_withdraw_fee_partner_token_amount = instant_withdraw_fee_partner.multiply(instant_withdraw_fee_self_token_amount);
                    if instant_withdraw_fee_partner_token_amount != 0 {
                        instant_withdraw_fee_self_token_amount -= instant_withdraw_fee_partner_token_amount;

                        match self.fungible_token.account_registry.get(&self.account_registry.partner_fee_receiver_account_id) {
                            Some(mut token_balance_) => {
                                token_balance_ += instant_withdraw_fee_partner_token_amount;

                                self.fungible_token.account_registry.insert(&self.account_registry.partner_fee_receiver_account_id, &token_balance_);
                            }
                            None => {
                                env::panic_str("Nonexecutable code. Object must exist.");
                            }
                        }
                    }
                }

                match self.fungible_token.account_registry.get(&self.account_registry.self_fee_receiver_account_id) {
                    Some(mut token_balance_) => {
                        token_balance_ += instant_withdraw_fee_self_token_amount;

                        self.fungible_token.account_registry.insert(&self.account_registry.self_fee_receiver_account_id, &token_balance_);
                    }
                    None => {
                        env::panic_str("Nonexecutable code. Object must exist.");
                    }
                }
            }
        }

        let mut near_amount = self.convert_token_amount_to_near_amount(token_amount);      // TODO  TODO  TODO  TODO  TODO  Может, конвертацию везде нужно считать на коллбеке в контексте лостАпдейта? братить внимание, что условия на проверку на ноль после конвертации уже ставиь не нужно, если решен вопрос с конвертацией.

        if near_amount == 0 {
            env::panic_str("Insufficient token amount.");
        }

        if near_amount > self.fund.classic_unstaked_balance {
            env::panic_str("Token amount exceeded the available unstaked near balance.");
        }

        self.fund.classic_unstaked_balance -= near_amount;

        let storage_staking_price_per_additional_account = if token_balance > 0
            || predecessor_account_id == self.account_registry.self_fee_receiver_account_id
            || predecessor_account_id == self.account_registry.partner_fee_receiver_account_id {
            self.fungible_token.account_registry.insert(&predecessor_account_id, &token_balance);

            0
        } else {
            self.fungible_token.account_registry.remove(&predecessor_account_id);
            self.fungible_token.accounts_quantity -= 1;

            let storage_staking_price_per_additional_account_ = Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_account);

            near_amount += storage_staking_price_per_additional_account_;

            storage_staking_price_per_additional_account_
        };

        self.fungible_token.total_supply -= token_amount;

        let attached_deposit = env::attached_deposit();

        near_amount += attached_deposit;

        let current_account_id = env::current_account_id();
        env::log_str(
            format!(
                "
                Instant withdrawing from @{} in {} epoch.
                Attached deposit is {} yoctoNear.
                Exchangeable deposit is {} yoctoStNear.
                Fee is {:?}.
                Released storage staking price is {} yoctoNear.
                Received amount is {} yoctoNear.
                Old @{} total supply is {} yoctoStNear.
                Old @{} balance is {} yoctoNear.
                Old @{} balance is {} yoctoStNear.
                New @{} balance is {} yoctoStNear.
                New @{} balance is {} yoctoNear.
                New @{} total supply is {} yoctoStNear.
                ",
                &current_account_id,
                self.current_epoch_height,
                attached_deposit,
                token_amount_,
                instant_withdraw_fee_self,
                storage_staking_price_per_additional_account,
                near_amount,
                &current_account_id,
                self.fungible_token.total_supply + token_amount,
                &current_account_id,
                self.fund.get_common_balance() + near_amount - storage_staking_price_per_additional_account - attached_deposit,
                &predecessor_account_id,
                token_balance_,
                &predecessor_account_id,
                token_balance,
                &current_account_id,
                self.fund.get_common_balance(),
                &current_account_id,
                self.fungible_token.total_supply
            ).as_str()
        );

        Promise::new(predecessor_account_id)
            .transfer(near_amount)
    }

    fn internal_delayed_withdraw(&mut self, token_amount: Balance) -> PromiseOrValue<()> {
        Self::assert_gas_is_enough();
        Self::assert_natural_deposit();
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

        if near_amount > self.fund.classic_staked_balance {
            env::panic_str("Token amount exceeded the available staked near balance.");
        }

        self.fund.classic_staked_balance -= near_amount;

        let attached_deposit = env::attached_deposit();

        let (mut near_refundable_deposit, mut delayed_withdrawal) = match self.fund.delayed_withdrawn_fund.delayed_withdrawal_registry.get(&predecessor_account_id) {
            Some(mut delayed_withdrawal_) => {
                delayed_withdrawal_.started_epoch_height = env::epoch_height();

                (attached_deposit, delayed_withdrawal_)
            }
            None => {
                let storage_staking_price_per_additional_delayed_withdrawal = Self::calculate_storage_staking_price(self.fund.delayed_withdrawn_fund.storage_usage_per_delayed_withdrawal);
                if attached_deposit < storage_staking_price_per_additional_delayed_withdrawal {
                    env::panic_str("Insufficient near deposit.");
                }

                (
                    attached_deposit - storage_staking_price_per_additional_delayed_withdrawal,
                    DelayedWithdrawal {
                        near_amount: 0,
                        started_epoch_height: env::epoch_height()
                    }
                )
            }
        };
        delayed_withdrawal.near_amount += near_amount;
        self.fund.delayed_withdrawn_fund.delayed_withdrawal_registry.insert(&predecessor_account_id, &delayed_withdrawal);
        self.fund.delayed_withdrawn_fund.needed_to_request_classic_near_amount += near_amount;

        token_balance -= token_amount;
        if let Some(investor_investment) = self.validating.investor_investment_registry.get(&predecessor_account_id) {
            if self.convert_token_amount_to_near_amount(token_balance) < investor_investment.staked_balance {
                env::panic_str("Token amount exceeded the available to delayed withdraw token amount.");
            }
        }
        if token_balance > 0
            || predecessor_account_id == self.account_registry.self_fee_receiver_account_id
            || predecessor_account_id == self.account_registry.partner_fee_receiver_account_id  {
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
// если вс. сумму с валидатора снимают, то уничтожается ли аккаунт и добавляется ли платеж сумма стораджа
    fn internal_delayed_withdraw_from_validator(&mut self, near_amount: Balance, validator_account_id: AccountId) -> PromiseOrValue<()> {
        Self::assert_gas_is_enough();
        Self::assert_natural_deposit();
        self.assert_epoch_is_synchronized();

        if near_amount == 0 {
            env::panic_str("Insufficient near amount.");
        }
        if near_amount > self.fund.investment_staked_balance {
            env::panic_str("Token amount exceeded the available staked near balance.");
        }

        let validator = match self.validating.validator_registry.get(&validator_account_id) {
            Some(validator_) => validator_,
            None => {
                env::panic_str("Validator account is not registered yet.");
            }
        };

        let predecessor_account_id = env::predecessor_account_id();

        let mut investor_investment = match self.validating.investor_investment_registry.get(&predecessor_account_id) {
            Some(investor_investment_) => investor_investment_,
            None => {
                env::panic_str("Investor account is not registered yet.");
            }
        };

        let mut staked_balance = match investor_investment.distribution_registry.get(&validator_account_id) {
            Some(staked_balance_) => staked_balance_,
            None => {
                env::panic_str("There is no investor stake on this validator.");
            }
        };
        if near_amount > staked_balance {
            env::panic_str("Near amount exceeded the available investor near balance on validator.");
        }

        let attached_deposit = env::attached_deposit();

        let (mut near_refundable_deposit, mut investment_withdrawal) =
            match self.fund.delayed_withdrawn_fund.investment_withdrawal_registry.get(&validator_account_id) {
            Some(investment_withdrawal_) => (attached_deposit, investment_withdrawal_),
            None => {
                let storage_staking_price_per_additional_investment_withdrawal =
                    Self::calculate_storage_staking_price(self.fund.delayed_withdrawn_fund.storage_usage_per_investment_withdrawal);
                if attached_deposit < storage_staking_price_per_additional_investment_withdrawal {
                    env::panic_str("Insufficient near deposit.");
                }

                (
                    attached_deposit - storage_staking_price_per_additional_investment_withdrawal,
                    InvestmentWithdrawal {
                        near_amount: 0,
                        account_id: predecessor_account_id.clone()
                    }
                )
            }
        };
        if near_amount > (validator.investment_staked_balance - investment_withdrawal.near_amount) {
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

        self.fund.investment_staked_balance -= near_amount;
        let mut delayed_withdrawal = match self.fund.delayed_withdrawn_fund.delayed_withdrawal_registry.get(&predecessor_account_id) {
            Some(mut delayed_withdrawal_) => {
                delayed_withdrawal_.started_epoch_height = env::epoch_height();

                delayed_withdrawal_
            }
            None => {
                let storage_staking_price_per_additional_delayed_withdrawal =
                    Self::calculate_storage_staking_price(self.fund.delayed_withdrawn_fund.storage_usage_per_delayed_withdrawal);
                if near_refundable_deposit < storage_staking_price_per_additional_delayed_withdrawal {
                    env::panic_str("Insufficient near deposit.");
                }
                near_refundable_deposit -= storage_staking_price_per_additional_delayed_withdrawal;

                DelayedWithdrawal {
                    near_amount: 0,
                    started_epoch_height: env::epoch_height()
                }
            }
        };
        delayed_withdrawal.near_amount += near_amount;
        self.fund.delayed_withdrawn_fund.delayed_withdrawal_registry.insert(&predecessor_account_id, &delayed_withdrawal);

        investment_withdrawal.near_amount += near_amount;
        self.fund.delayed_withdrawn_fund.investment_withdrawal_registry.insert(&validator_account_id, &investment_withdrawal);
        self.fund.delayed_withdrawn_fund.needed_to_request_investment_near_amount += near_amount;

        if near_amount < staked_balance {
            staked_balance -= near_amount;

            investor_investment.distribution_registry.insert(&validator_account_id, &staked_balance);
        } else {
            investor_investment.distribution_registry.remove(&validator_account_id);
            investor_investment.distributions_quantity -= 1;

            near_refundable_deposit += Self::calculate_storage_staking_price(self.validating.storage_usage_per_distribution);
        }
        investor_investment.staked_balance -= near_amount;
        self.validating.investor_investment_registry.insert(&predecessor_account_id, &investor_investment);

        token_balance -= token_amount;
        if token_balance > 0
            || predecessor_account_id == self.account_registry.self_fee_receiver_account_id
            || predecessor_account_id == self.account_registry.partner_fee_receiver_account_id  {
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
        Self::assert_natural_deposit();
        self.assert_epoch_is_synchronized();

        let predecessor_account_id = env::predecessor_account_id();

        let delayed_withdrawal = match self.fund.delayed_withdrawn_fund.delayed_withdrawal_registry.remove(&predecessor_account_id) {
            Some(delayed_withdrawal_) => delayed_withdrawal_,
            None => {
                env::panic_str("Delayed withdrawal account is not registered.");
            }
        };
        if !delayed_withdrawal.can_take_delayed_withdrawal(self.current_epoch_height) {
            env::panic_str("Wrong epoch for withdrawal.");
        }

        self.fund.delayed_withdrawn_fund.balance -= delayed_withdrawal.near_amount;

        let near_amount = delayed_withdrawal.near_amount
            + Self::calculate_storage_staking_price(self.fund.delayed_withdrawn_fund.storage_usage_per_delayed_withdrawal)
            + env::attached_deposit();

        Promise::new(predecessor_account_id)
            .transfer(near_amount)
    }

    fn internal_increase_validator_stake(&mut self, validator_account_id: AccountId, near_amount: Balance) -> Promise {      // TODO Сюда нужно зафиксировать максимальное число Газа. Возможно ли?
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        if near_amount == 0 {
            env::panic_str("Insufficient near amount.");
        }

        if self.fund.classic_unstaked_balance == 0
            || !(1..=self.fund.classic_unstaked_balance).contains(&near_amount) {
                env::panic_str("Near amount exceeded the available unstaked near balance.");
        }

        match self.validating.validator_registry.get(&validator_account_id) {
            Some(validator) => {                                                                       // TODO написать все везде в одном стиле let vi = match ... То есть, убрать вложенность там, где возможно
                if validator.is_only_for_investment {
                    env::panic_str("Validator is used only for investment purpose.");
                }

                match validator.staking_contract_version {
                    StakingContractVersion::Classic => {
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
        if !Self::is_right_epoch(env::epoch_height()) {
            env::panic_str("Epoch is not intended for a requested decrease validator stake request.");
        }

        if near_amount == 0 {
            env::panic_str("Insufficient near amount.");
        }

        let validator = match self.validating.validator_registry.get(&validator_account_id) {     // TODO проверить на правильные ли методы идут кроссколы. Взять весб баланс или взять анстейкед баланс или взять стейкед баланс.
            Some(validator_) => validator_,
            None => {
                env::panic_str("Validator account is not registered yet.");
            }
        };
        match stake_decreasing_type {
            StakeDecreasingType::Classic => {
                if near_amount > validator.classic_staked_balance {
                    env::panic_str("Near amount exceeded the available staked near balance.");
                }
                if near_amount > self.fund.delayed_withdrawn_fund.needed_to_request_classic_near_amount {
                    env::panic_str("Near amount is more than requested near amount.");
                }
            }
            StakeDecreasingType::Investment => {
                if near_amount > validator.investment_staked_balance {
                    env::panic_str("Near amount exceeded the available unstaked near balance.");
                }
                if near_amount > self.fund.delayed_withdrawn_fund.needed_to_request_investment_near_amount {
                    env::panic_str("Near amount is more than requested near amount.");
                }

                let investment_withdrawal = match self.fund.delayed_withdrawn_fund.investment_withdrawal_registry.get(&validator_account_id) {
                    Some(investment_withdrawal_) => investment_withdrawal_,
                    None => {
                        env::panic_str("Investment withdrawal account is not registered yet.");
                    }
                };
                if near_amount > investment_withdrawal.near_amount {
                    env::panic_str("Near amount is more than requested near amount from validator.");
                }
            }
        }

        match validator.staking_contract_version {
            StakingContractVersion::Classic => {
                validator::ext(validator_account_id.clone())
                    // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                    .unstake(near_amount.into())
                    .then(
                        Self::ext(env::current_account_id())
                            .requested_decrease_validator_stake_callback(
                                validator_account_id,
                                near_amount,
                                stake_decreasing_type,
                                Self::calculate_storage_staking_price(self.fund.delayed_withdrawn_fund.storage_usage_per_investment_withdrawal)
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

        if !Self::is_right_epoch(current_epoch_height) {
            env::panic_str("Epoch is not intended for a take unstaked balance.");
        }

        match self.validating.validator_registry.get(&validator_account_id) {   // TODO // TODO ЧТо будет, если валидатор перестал работать, что придет с контракта. Не прервется ли из-за этго цепочка выполнения апдейтов
            Some(validator) => {
                if validator.unstaked_balance == 0 {
                    env::panic_str("Insufficient unstaked balance on validator.");
                }
                if validator.last_update_epoch_height >= current_epoch_height {       // TODO нужно ли это услвоие
                    env::panic_str("Validator is already updated.");
                }

                match validator.staking_contract_version {
                    StakingContractVersion::Classic => {
                        validator::ext(validator_account_id.clone())
                            // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                            .withdraw(validator.unstaked_balance.into())
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

    fn internal_update_validator(&mut self, validator_account_id: AccountId) -> Promise {     // TODO TODO TODO Что делать, если в новой эпохе часть обновилась, и уже еще раз наступила новая эпоха, и теперь то, что осталось, обновились. То есть, рассинхронизация состояния.   // TODO Сюда нужно зафиксировать максимальное число Газа. Возможно ли?
        Self::assert_gas_is_enough();
        self.assert_epoch_is_desynchronized();
        self.assert_authorized_management_only_by_manager();

        match self.validating.validator_registry.get(&validator_account_id) {   // TODO // TODO ЧТо будет, если валидатор перестал работать, что придет с контракта. Не прервется ли из-за этго цепочка выполнения апдейтов
            Some(validator) => {
                let current_epoch_height = env::epoch_height();

                if validator.last_update_epoch_height < current_epoch_height {
                    match validator.staking_contract_version {
                        StakingContractVersion::Classic => {
                            validator::ext(validator_account_id.clone())
                                // .with_static_gas(deposit_and_stake_gas)                  // CCX выполняется, если прикрепить меньше, чем нужно, но выпролняться не должен.
                                .get_account_total_balance(env::current_account_id())
                                .then(
                                    Self::ext(env::current_account_id())
                                        .update_validator_callback(validator_account_id, current_epoch_height)
                                )
                        }
                    }
                } else {
                    env::panic_str("Validator is already updated.");
                }
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

        let current_epoch_height = env::epoch_height();

        if self.validating.validators_quantity > 0 {
            if (self.validating.quantity_of_validators_updated_in_current_epoch / self.validating.validators_quantity == 0)
                || (self.validating.quantity_of_validators_updated_in_current_epoch % self.validating.validators_quantity != 0) {
                env::panic_str("Some validators are not updated.");
            }

            if Self::is_right_epoch(current_epoch_height)
                && (self.fund.delayed_withdrawn_fund.needed_to_request_classic_near_amount > 0
                    || self.fund.delayed_withdrawn_fund.needed_to_request_investment_near_amount > 0) {
                    env::panic_str("Some funds are not unstaked from validators.");
            }

            self.fund.classic_staked_balance += self.previous_epoch_rewards_from_validators_near_amount;
            self.validating.quantity_of_validators_updated_in_current_epoch = 0;
            self.total_rewards_from_validators_near_amount += self.previous_epoch_rewards_from_validators_near_amount;     // TODO переназвать, Убрать в впомагательные параметры.

            let previous_epoch_rewards_from_validators_token_amount = self.convert_near_amount_to_token_amount(
                self.previous_epoch_rewards_from_validators_near_amount
            );

            if let Some(ref reward_fee) = self.fee_registry.reward_fee {
                let mut reward_fee_self_token_amount = reward_fee.self_fee.multiply(previous_epoch_rewards_from_validators_token_amount);
                if reward_fee_self_token_amount != 0 {
                    self.fungible_token.total_supply += reward_fee_self_token_amount;

                    if let Some(ref reward_fee_partner) = reward_fee.partner_fee {
                        let reward_fee_partner_token_amount = reward_fee_partner.multiply(reward_fee_self_token_amount);
                        if reward_fee_partner_token_amount != 0 {
                            reward_fee_self_token_amount -= reward_fee_partner_token_amount;

                            match self.fungible_token.account_registry.get(&self.account_registry.partner_fee_receiver_account_id) {
                                Some(mut token_balance) => {
                                    token_balance += reward_fee_partner_token_amount;

                                    self.fungible_token.account_registry.insert(&self.account_registry.partner_fee_receiver_account_id, &token_balance);
                                }
                                None => {
                                    env::panic_str("Nonexecutable code. Object must exist.");
                                }
                            }
                        }
                    }

                    match self.fungible_token.account_registry.get(&self.account_registry.self_fee_receiver_account_id) {
                        Some(mut token_balance) => {
                            token_balance += reward_fee_self_token_amount;

                            self.fungible_token.account_registry.insert(&self.account_registry.self_fee_receiver_account_id, &token_balance);
                        }
                        None => {
                            env::panic_str("Nonexecutable code. Object must exist.");
                        }
                    }
                }
            }

            self.previous_epoch_rewards_from_validators_near_amount = 0;
        }

        self.fund.is_distributed_on_validators_in_current_epoch = false;
        self.current_epoch_height = current_epoch_height;
    }

    fn internal_add_validator(
        &mut self,
        validator_account_id: AccountId,
        staking_contract_version: StakingContractVersion,
        is_only_for_investment: bool,
        is_preferred: bool
    ) -> PromiseOrValue<()> {                                                     // TODO можно ли проверить, что адрес валиден, и валидатор в вайт-листе?
        Self::assert_gas_is_enough();
        Self::assert_natural_deposit();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        if is_preferred && is_only_for_investment {
            env::panic_str("Prefferred validator can not be only for investment.");
        }

        let attached_deposit = env::attached_deposit();

        let storage_staking_price_per_additional_validator = Self::calculate_storage_staking_price(self.validating.storage_usage_per_validator);
        if attached_deposit < storage_staking_price_per_additional_validator {
            env::panic_str("Insufficient near deposit.");
        }

        if let Some(_) = self.validating.validator_registry.insert(
            &validator_account_id, &Validator::new(staking_contract_version, is_only_for_investment)
        ) {
            env::panic_str("Validator account is already registered.");
        }
        self.validating.validators_quantity += 1;

        if is_preferred {
            self.validating.preffered_validator = Some(validator_account_id);
        }

        let near_amount = attached_deposit - storage_staking_price_per_additional_validator;
        if near_amount > 0 {
            return PromiseOrValue::Promise(
                Promise::new(env::predecessor_account_id())
                    .transfer(near_amount)
            );
        }

        PromiseOrValue::Value(())
    }

    fn internal_remove_validator(&mut self, validator_account_id: AccountId) -> Promise {       // TODO На всякий слуяай проверять, есть ли на валидаторе средстыва пула через кроссколл.
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        let validator = match self.validating.validator_registry.remove(&validator_account_id) {
            Some(validator_) => validator_,
            None => {
                env::panic_str("Validator account is not registered yet.");
            }
        };
        if validator.classic_staked_balance > 0
            || validator.investment_staked_balance > 0
            || validator.unstaked_balance > 0 {       // TODO  TODO TODO TODO TODO подумать, при каких условиях еще невозможно удалить валидатор.
            env::panic_str("Validator has an available balance.");
        }

        self.validating.validators_quantity -= 1;

        if let Some(ref preffered_validator_account_id) = self.validating.preffered_validator {
            if *preffered_validator_account_id == validator_account_id {
                self.validating.preffered_validator = None;
            }
        }

        let refundable_near_amount = Self::calculate_storage_staking_price(self.validating.storage_usage_per_validator);

        Promise::new(env::predecessor_account_id())
            .transfer(refundable_near_amount)
    }

    fn internal_change_validator_investment_context(&mut self, validator_account_id: AccountId, is_only_for_investment: bool) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        let mut validator = match self.validating.validator_registry.get(&validator_account_id) {
            Some(validator_) => validator_,
            None => {
                env::panic_str("Validator account is not registered yet.");
            }
        };

        if validator.is_only_for_investment == is_only_for_investment {
            env::panic_str("Changing the state to the same state.");
        }

        if is_only_for_investment {
            if let Some(ref preffered_validator_account_id) = self.validating.preffered_validator {
                if *preffered_validator_account_id == validator_account_id {
                    env::panic_str("Prefferred validator can not be only for investment.");
                }
            }

            if validator.classic_staked_balance > 0 {
                env::panic_str("Validator classic staked balance is not equal to zero.");
            }
        }

        validator.is_only_for_investment = is_only_for_investment;
        self.validating.validator_registry.insert(&validator_account_id, &validator);
    }

    fn internal_change_preffered_validator(&mut self, validator_account_id: Option<AccountId>) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        match validator_account_id {
            Some(validator_account_id_) => {
                let validator = match self.validating.validator_registry.get(&validator_account_id_) {
                    Some(validator_) => validator_,
                    None => {
                        env::panic_str("Validator account is not registered yet.");
                    }
                };

                if let Some(ref preffered_validator_account_id) = self.validating.preffered_validator {
                    if *preffered_validator_account_id == validator_account_id_ {
                        env::panic_str("Changing the state to the same state.");
                    }
                }

                if validator.is_only_for_investment {
                    env::panic_str("Prefferred validator can not be only for investment.");
                }

                self.validating.preffered_validator = Some(validator_account_id_);
            }
            None => {
                if let None = self.validating.preffered_validator {
                    env::panic_str("Changing the state to the same state.");
                }

                self.validating.preffered_validator = None;
            }
        }
    }

    fn internal_add_investor(&mut self, investor_account_id: AccountId) -> PromiseOrValue<()> {
        Self::assert_gas_is_enough();
        Self::assert_natural_deposit();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        let storage_staking_price_per_additional_investor_investment = Self::calculate_storage_staking_price(self.validating.storage_usage_per_investor_investment);
        if env::attached_deposit() < storage_staking_price_per_additional_investor_investment {
            env::panic_str("Insufficient near deposit.");
        }

        if let Some(_) = self.validating.investor_investment_registry.insert(
            &investor_account_id, &InvestorInvestment::new(investor_account_id.clone())
        ) {
            env::panic_str("Investor account is already registered.");
        }

        let near_amount = env::attached_deposit() - storage_staking_price_per_additional_investor_investment;
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

        let investor_investment = match self.validating.investor_investment_registry.remove(&investor_account_id) {  // TODO TODO TODO TODO TODO Обратить внимание, что на постоянное количество инвестмент токенов приходится увеличивающееся количество неара, но инвестмент баланс зафиксирован.
            Some(investor_investment_) => investor_investment_,
            None => {
                env::panic_str("Investor account is not registered yet.");
            }
        };
        if investor_investment.staked_balance > 0 || investor_investment.distributions_quantity > 0 {
            env::panic_str("Validator has an available balance.");
        }

        let near_amount = Self::calculate_storage_staking_price(self.validating.storage_usage_per_investor_investment);

        Promise::new(env::predecessor_account_id())
            .transfer(near_amount)
    }

    fn internal_change_manager(&mut self, manager_id: AccountId) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management();

        self.account_registry.manager_id = manager_id;
    }

    fn internal_change_reward_fee(&mut self, reward_fee_self: Option<Fee>, reward_fee_partner: Option<Fee>) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        if reward_fee_self.is_none() && reward_fee_partner.is_some() {
            env::panic_str("Reward fees are not valid.");
        }
        self.fee_registry.reward_fee = if let Some(reward_fee_self_) = reward_fee_self {
            reward_fee_self_.assert_valid();

            if let Some(ref reward_fee_partner) = reward_fee_partner {
                reward_fee_partner.assert_valid();
            }

            Some (
                SharedFee {
                    self_fee: reward_fee_self_,
                    partner_fee: reward_fee_partner
                }
            )
        } else {
            None
        };
    }

    fn internal_change_instant_withdraw_fee(&mut self, instant_withdraw_fee_self: Option<Fee>, instant_withdraw_fee_partner: Option<Fee>) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        if instant_withdraw_fee_self.is_none() && instant_withdraw_fee_partner.is_some() {
            env::panic_str("Instant withdraw fees are not valid.");
        }
        self.fee_registry.instant_withdraw_fee = if let Some(instant_withdraw_fee_self_) = instant_withdraw_fee_self {
            instant_withdraw_fee_self_.assert_valid();

            if let Some(ref instant_withdraw_fee_partner) = instant_withdraw_fee_partner {
                instant_withdraw_fee_partner.assert_valid();
            }

            Some (
                SharedFee {
                    self_fee: instant_withdraw_fee_self_,
                    partner_fee: instant_withdraw_fee_partner
                }
            )
        } else {
            None
        };
    }

    fn internal_confirm_stake_distribution(&mut self) {
        Self::assert_gas_is_enough();
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        if self.fund.is_distributed_on_validators_in_current_epoch {
            env::panic_str("Fund has already been distributed.");
        }

        self.fund.is_distributed_on_validators_in_current_epoch = true;
    }

    fn internal_ft_transfer(&mut self, receiver_account_id: AccountId, token_amount: Balance) -> Promise {     // TODO стоит ли удалять токен-аккаунт с нулевым балансом. Сейчас я его удаляю. нет механизма регистрации. Сейчас регистрируется тоько при депозите
        Self::assert_gas_is_enough();
        Self::assert_natural_deposit();

        if token_amount == 0 {
            env::panic_str("Insufficient token amount.");
        }

        let mut refundable_near_amount = env::attached_deposit();

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

        if let Some(investor_investment) = self.validating.investor_investment_registry.get(&predecessor_account_id) {
            if self.convert_token_amount_to_near_amount(predecessor_account_token_balance) < investor_investment.staked_balance {
                env::panic_str("Token amount exceeded the available to transfer token amount.");
            }
        }

        receiver_account_token_balance += token_amount;

        if predecessor_account_token_balance > 0
            || predecessor_account_id == self.account_registry.self_fee_receiver_account_id
            || predecessor_account_id == self.account_registry.partner_fee_receiver_account_id {                                           // TODO стоит ли записать здесь и везде два последних условия через метод
            self.fungible_token.account_registry.insert(&predecessor_account_id, &predecessor_account_token_balance);
        } else {
            self.fungible_token.account_registry.remove(&predecessor_account_id);
            self.fungible_token.accounts_quantity -= 1;

            refundable_near_amount += Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_account);
        }
        self.fungible_token.account_registry.insert(&receiver_account_id, &receiver_account_token_balance);

        Promise::new(predecessor_account_id)
            .transfer(refundable_near_amount)
    }

    pub fn internal_get_delayed_withdrawal_details(&self, account_id: AccountId) -> Option<DelayedWithdrawalDetails> {
        self.assert_epoch_is_synchronized();

        if let Some(delayed_withdrawal) = self.fund.delayed_withdrawn_fund.delayed_withdrawal_registry.get(&account_id) {
            return Some(
                    DelayedWithdrawalDetails {
                        epoch_quantity_to_take_delayed_withdrawal: delayed_withdrawal.get_epoch_quantity_to_take_delayed_withdrawal(self.current_epoch_height),
                        near_amount: delayed_withdrawal.near_amount.into()
                }
            );
        }

        None
    }

    fn internal_get_total_token_supply(&self) -> Balance {
        self.assert_epoch_is_synchronized();

        self.fungible_token.total_supply
    }

    fn internal_get_minimum_deposit_amount(&self) -> Balance {
        MINIMUN_DEPOSIT_AMOUNT
    }

    pub fn internal_get_storage_staking_price(&self) -> StorageStakingPrice {
        StorageStakingPrice {
            per_delayed_withdrawal_fund_delayed_withdrawal: Self::calculate_storage_staking_price(self.fund.delayed_withdrawn_fund.storage_usage_per_delayed_withdrawal).into(),
            per_delayed_withdrawal_fund_investment_withdrawal: Self::calculate_storage_staking_price(self.fund.delayed_withdrawn_fund.storage_usage_per_investment_withdrawal).into(),
            per_fungible_token_account: Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_account).into(),
            per_validating_node_validator: Self::calculate_storage_staking_price(self.validating.storage_usage_per_validator).into(),
            per_validating_node_investor: Self::calculate_storage_staking_price(self.validating.storage_usage_per_investor_investment).into(),
            per_validating_node_distribution: Self::calculate_storage_staking_price(self.validating.storage_usage_per_distribution).into()
        }
    }

    pub fn internal_get_storage_staking_requested_coverage(&self, account_id: AccountId) -> StorageStakingRequestedCoverage {
        self.assert_epoch_is_synchronized();

        let storage_staking_price_per_delayed_withdrawal_fund_delayed_withdrawal = Self::calculate_storage_staking_price(self.fund.delayed_withdrawn_fund.storage_usage_per_delayed_withdrawal);

        let storage_staking_price_per_delayed_withdrawal_fund_investment_withdrawal = Self::calculate_storage_staking_price(self.fund.delayed_withdrawn_fund.storage_usage_per_investment_withdrawal);

        let storage_staking_price_per_fungible_token_account = Self::calculate_storage_staking_price(self.fungible_token.storage_usage_per_account);

        let storage_staking_price_per_validating_node_distribution = Self::calculate_storage_staking_price(self.validating.storage_usage_per_distribution);

        let per_method_deposit = if !self.fungible_token.account_registry.contains_key(&account_id) {
            storage_staking_price_per_fungible_token_account
        } else {
            0
        };

        let per_method_delayed_withdraw = if !self.fund.delayed_withdrawn_fund.delayed_withdrawal_registry.contains_key(&account_id) {
            storage_staking_price_per_delayed_withdrawal_fund_delayed_withdrawal
        } else {
            0
        };

        let (per_method_deposit_on_validator, per_method_delayed_withdraw_from_validator): (Option<(U128, Vec<(AccountId, U128)>)>, Option<(U128, Vec<(AccountId, U128)>)>) =
            match self.validating.investor_investment_registry.get(&account_id) {
            Some(investor_investment) => {
                let requested_storage_staking_price_per_fungible_token_account = if !self.fungible_token.account_registry.contains_key(&account_id) {
                    storage_staking_price_per_fungible_token_account
                } else {
                    0
                };

                let requested_storage_staking_price_per_delayed_withdrawal_fund_delayed_withdrawal = if !self.fund.delayed_withdrawn_fund.delayed_withdrawal_registry.contains_key(&account_id) {
                    storage_staking_price_per_delayed_withdrawal_fund_delayed_withdrawal
                } else {
                    0
                };

                let mut requested_storage_staking_price_per_distribution_registry: Vec<(AccountId, U128)> = vec![];

                let mut requested_storage_staking_price_per_delayed_withdrawal_fund_investment_withdrawal_registry: Vec<(AccountId, U128)> = vec![];

                for validator_account_id in self.validating.validator_registry.keys() {
                    if !investor_investment.distribution_registry.contains_key(&validator_account_id) {
                        requested_storage_staking_price_per_distribution_registry.push((validator_account_id, storage_staking_price_per_validating_node_distribution.into()));
                    } else {
                        requested_storage_staking_price_per_distribution_registry.push((validator_account_id.clone(), 0.into()));

                        if !self.fund.delayed_withdrawn_fund.investment_withdrawal_registry.contains_key(&validator_account_id) {
                            requested_storage_staking_price_per_delayed_withdrawal_fund_investment_withdrawal_registry.push((validator_account_id, storage_staking_price_per_delayed_withdrawal_fund_investment_withdrawal.into()));
                        } else {
                            requested_storage_staking_price_per_delayed_withdrawal_fund_investment_withdrawal_registry.push((validator_account_id, 0.into()));
                        }
                    }
                }

                (
                    Some((requested_storage_staking_price_per_fungible_token_account.into(), requested_storage_staking_price_per_distribution_registry)),
                    Some((requested_storage_staking_price_per_delayed_withdrawal_fund_delayed_withdrawal.into(), requested_storage_staking_price_per_delayed_withdrawal_fund_investment_withdrawal_registry))
                )
            }
            None => {
                (None, None)
            }
        };

        StorageStakingRequestedCoverage {
            per_method_deposit: per_method_deposit.into(),
            per_method_deposit_on_validator,
            per_method_delayed_withdraw: per_method_delayed_withdraw.into(),
            per_method_delayed_withdraw_from_validator
        }
    }

    fn internal_get_account_balance(&self, account_id: AccountId) -> AccountBalance {
        match self.fungible_token.account_registry.get(&account_id) {
            Some(token_balance) => {
                let common_near_balance = self.convert_token_amount_to_near_amount(token_balance);

                match self.validating.investor_investment_registry.get(&account_id) {
                    Some(investor_investment) => {
                        if common_near_balance < investor_investment.staked_balance {
                            env::panic_str("Nonexecutable code. Near balance should be greater then or equal to investment near balance.");
                        }

                        AccountBalance {
                            base_account_balance: Some(
                                BaseAccountBalance {
                                    token_balance: token_balance.into(),
                                    common_near_balance: common_near_balance.into(),
                                    classic_near_balance: (common_near_balance - investor_investment.staked_balance).into(),
                                }
                            ),
                            investment_account_balance: Some(investor_investment.staked_balance.into())
                        }
                    }
                    None => {
                        AccountBalance {
                            base_account_balance: Some(
                                BaseAccountBalance {
                                    token_balance: token_balance.into(),
                                    common_near_balance: common_near_balance.into(),
                                    classic_near_balance: common_near_balance.into(),
                                }
                            ),
                            investment_account_balance: None
                        }
                    }
                }
            }
            None => {
                match self.validating.investor_investment_registry.get(&account_id) {
                    Some(investor_investment) => {
                        AccountBalance {
                            base_account_balance: None,
                            investment_account_balance: Some(investor_investment.staked_balance.into())
                        }
                    }
                    None => {
                        AccountBalance {
                            base_account_balance: None,
                            investment_account_balance: None
                        }
                    }
                }
            }
        }
    }

    fn internal_get_fund(&self) -> FundDto {
        self.assert_epoch_is_synchronized();

        FundDto {
            classic_unstaked_balance: self.fund.classic_unstaked_balance.into(),
            classic_staked_balance: self.fund.classic_staked_balance.into(),
            investment_staked_balance: self.fund.investment_staked_balance.into(),
            common_staked_balance: self.fund.get_staked_balance().into(),
            common_balance: self.fund.get_common_balance().into()
        }
    }

    fn internal_get_fee_registry(&self) -> FeeRegistry {
        self.assert_epoch_is_synchronized();
        self.assert_authorized_management_only_by_manager();

        self.fee_registry.clone()
    }

    fn internal_get_fee_registry_light(&self) -> FeeRegistryLight {
        self.assert_epoch_is_synchronized();

        let reward_fee = match self.fee_registry.reward_fee {
            Some(ref reward_fee_) => Some(reward_fee_.self_fee.clone()),
            None => None
        };

        let instant_withdraw_fee = match self.fee_registry.instant_withdraw_fee {
            Some(ref instant_withdraw_fee_) => Some(instant_withdraw_fee_.self_fee.clone()),
            None => None
        };

        FeeRegistryLight {
            reward_fee,
            instant_withdraw_fee
        }
    }

    pub fn internal_get_current_epoch_height(&self) -> EpochHeightRegistry {
        EpochHeightRegistry {
            pool_epoch_height: self.current_epoch_height,
            network_epoch_height: env::epoch_height()
        }
    }

    pub fn internal_is_stake_distributed(&self) -> bool {
        self.fund.is_distributed_on_validators_in_current_epoch
    }

    pub fn internal_get_investor_investment(&self, account_id: AccountId) -> Option<InvestorInvestmentDto> {
        self.assert_epoch_is_synchronized();

        let mut distribution_registry: Vec<(AccountId, U128)> = vec![];

        let investor_investment = match self.validating.investor_investment_registry.get(&account_id) {
            Some(investor_investment_) => investor_investment_,
            None => {
                return None;
            }
        };

        for validator_account_id in self.validating.validator_registry.keys() {
            if let Some(staked_balance) = investor_investment.distribution_registry.get(&validator_account_id) {
                distribution_registry.push((validator_account_id, staked_balance.into()));
            }
        }

        Some(
            InvestorInvestmentDto {
                distribution_registry,
                staked_balance: investor_investment.staked_balance.into()
            }
        )
    }

    fn internal_get_validator_registry(&self) -> Vec<ValidatorDto> {
        let mut validator_dto_registry: Vec<ValidatorDto> = vec![];

        for (account_id, validator) in self.validating.validator_registry.into_iter() {
            let Validator {
                unstaked_balance,
                classic_staked_balance,
                investment_staked_balance,
                staking_contract_version: _,
                is_only_for_investment,
                last_update_epoch_height,
                last_classic_stake_increasing_epoch_height
            } = validator;

            validator_dto_registry.push(
                ValidatorDto {
                    account_id,
                    unstaked_balance: unstaked_balance.into(),
                    classic_staked_balance: classic_staked_balance.into(),
                    investment_staked_balance: investment_staked_balance.into(),
                    is_only_for_investment,
                    last_update_epoch_height,
                    last_classic_stake_increasing_epoch_height
                }
            );
        }

        validator_dto_registry
    }

    fn internal_get_preffered_validator(&self) -> Option<ValidatorDto> {
        if let Some(ref preffered_validator_account_id) = self.validating.preffered_validator {
            let validator = match self.validating.validator_registry.get(preffered_validator_account_id) {
                Some(validator_) => validator_,
                None => {
                    env::panic_str("Nonexecutable code. Object must exist.");
                }
            };

            let Validator {
                unstaked_balance,
                classic_staked_balance,
                investment_staked_balance,
                staking_contract_version: _,
                is_only_for_investment,
                last_update_epoch_height,
                last_classic_stake_increasing_epoch_height
            } = validator;

            return Some(
                ValidatorDto {
                    account_id: preffered_validator_account_id.clone(),
                    unstaked_balance: unstaked_balance.into(),
                    classic_staked_balance: classic_staked_balance.into(),
                    investment_staked_balance: investment_staked_balance.into(),
                    is_only_for_investment,
                    last_update_epoch_height,
                    last_classic_stake_increasing_epoch_height
                }
            )
        }

        None
    }

    fn internal_get_aggregated(&self) -> Aggregated {
        self.assert_epoch_is_synchronized();

        let reward_fee_self = if let Some(ref reward_fee) = self.fee_registry.reward_fee {
            Some(reward_fee.self_fee.clone())
        } else {
            None
        };

        Aggregated {
            unstaked_balance: self.fund.classic_unstaked_balance.into(),
            staked_balance: self.fund.get_staked_balance().into(),
            token_total_supply: self.fungible_token.total_supply.into(),
            token_accounts_quantity: self.fungible_token.accounts_quantity,
            total_rewards_from_validators_near_amount: self.total_rewards_from_validators_near_amount.into(),
            reward_fee: reward_fee_self
        }
    }

    fn internal_get_requested_to_withdrawal_fund(&self) -> RequestedToWithdrawalFund {
        let mut investment_withdrawal_registry: Vec<(AccountId, U128)> = vec![];

        for validator_account_id in self.validating.validator_registry.keys() {
            if let Some(investment_withdrawal) = self.fund.delayed_withdrawn_fund.investment_withdrawal_registry.get(&validator_account_id) {
                investment_withdrawal_registry.push((validator_account_id, investment_withdrawal.near_amount.into()))
            }
        }

        RequestedToWithdrawalFund {
            classic_near_amount: self.fund.delayed_withdrawn_fund.needed_to_request_classic_near_amount.into(),
            investment_near_amount: self.fund.delayed_withdrawn_fund.needed_to_request_investment_near_amount.into(),
            investment_withdrawal_registry
        }
    }

    pub fn internal_get_full(&self) -> Full {
        self.assert_epoch_is_synchronized();

        Full {
            storage_staking_price: self.internal_get_storage_staking_price(),
            fund: self.internal_get_fund(),
            total_token_supply: self.internal_get_total_token_supply().into(),
            fee_registry_light: self.internal_get_fee_registry_light(),
            minimum_deposit_amount: self.get_minimum_deposit_amount()
        }
    }

    pub fn internal_get_full_for_account(&self, account_id: AccountId) -> FullForAccount {
        self.assert_epoch_is_synchronized();

        FullForAccount {
            full: self.internal_get_full(),
            account_balance: self.internal_get_account_balance(account_id.clone()),
            delayed_withdrawal_details: self.internal_get_delayed_withdrawal_details(account_id.clone()),
            investor_investment: self.get_investor_investment(account_id.clone()),
            storage_staking_requested_coverage: self.get_storage_staking_requested_coverage(account_id)
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
        let common_balance = self.fund.get_common_balance();

        if common_balance == 0 || near_amount == 0 {
            return near_amount;
        }

        (
            U256::from(near_amount)
            * U256::from(self.fungible_token.total_supply)
            / U256::from(common_balance)             // TODO Проверить Округление
        ).as_u128()
    }

    fn convert_token_amount_to_near_amount(&self, token_amount: Balance) -> Balance {      // TOD вот здесь обратить внимание. Правильно ли стоит проверка в случае, если здесь ноль, а неаров не ноль. ТАкое может быть в контексте получения и вывда ревардов
        if self.fungible_token.total_supply == 0 || token_amount == 0 {
            return token_amount
        }

        (
            U256::from(token_amount)
            * U256::from(self.fund.get_common_balance())             // TODO Проверить Округление
            / U256::from(self.fungible_token.total_supply)
        ).as_u128()
    }

    fn assert_authorized_management_only_by_manager(&self) {
        if env::predecessor_account_id() != self.account_registry.manager_id {
            env::panic_str("Unauthorized management. Management must be carried out either by the manager of the pool.");
        }
    }

    fn assert_authorized_management(&self) {
        let predecessor_account_id = env::predecessor_account_id();

        if predecessor_account_id != self.account_registry.owner_id
            && predecessor_account_id != self.account_registry.manager_id {
            env::panic_str("Unauthorized management. Management must be carried out either by the owner or manager of the pool.");
        }
    }

    fn assert_natural_deposit() {
        if env::attached_deposit() == 0 {
            env::panic_str("Not natural attached deposit.");
        }
    }

    fn assert_minimum_deposit() {
        if env::attached_deposit() < MINIMUN_DEPOSIT_AMOUNT {
            env::panic_str("Attached deposit less then minimum required deposit.");
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

    fn is_right_epoch(epoch_height: EpochHeight) -> bool {
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
        attached_deposit: Balance,
        near_amount: Balance,
        refundable_near_amount: Balance,
        token_amount: Balance,
        current_epoch_height: EpochHeight,
        storage_staking_price_per_additional_account: Balance
    ) {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let mut validator = match self.validating.validator_registry.get(&validator_account_id) {
                    Some(validator_) => validator_,
                    None => {
                        env::panic_str("Nonexecutable code. Object must exist.");
                    }
                };
                validator.classic_staked_balance += near_amount;
                validator.last_classic_stake_increasing_epoch_height = Some(current_epoch_height);
                self.validating.validator_registry.insert(&validator_account_id, &validator);

                self.fund.classic_staked_balance += near_amount;
            }
            _ => {
                self.fund.classic_unstaked_balance += near_amount;
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

        if refundable_near_amount > 0 {
            Promise::new(predecessor_account_id.clone())
                .transfer(refundable_near_amount);
        }

        let current_account_id = env::current_account_id();
        env::log_str(
            format!(
                "
                Deposited to @{} via @{} in {} epoch.
                Attached deposit is {} yoctoNear.
                Exchangeable deposit is {} yoctoNear.
                Reserved storage staking price is {} yoctoNear.
                Refundable deposit is {} yoctoNear.
                Old @{} total supply is {} yoctoStNear.
                Old @{} balance is {} yoctoNear.
                Old @{} balance is {} yoctoStNear.
                @{} received {} yoctoStNear.
                New @{} balance is {} yoctoStNear.
                New @{} balance is {} yoctoNear.
                New @{} total supply is {} yoctoStNear.
                ",
                &validator_account_id,
                &current_account_id,
                env::epoch_height(),
                attached_deposit,
                near_amount,
                storage_staking_price_per_additional_account,
                refundable_near_amount,
                &current_account_id,
                self.fungible_token.total_supply - token_amount,
                &current_account_id,
                self.fund.get_common_balance() - near_amount,
                &predecessor_account_id,
                token_balance - token_amount,
                &predecessor_account_id,
                token_amount,
                &predecessor_account_id,
                token_balance,
                &current_account_id,
                self.fund.get_common_balance(),
                &current_account_id,
                self.fungible_token.total_supply
            ).as_str()
        );
    }

    #[private]
    pub fn deposit_on_validator_callback(
        &mut self,
        predecessor_account_id: AccountId,
        validator_account_id: AccountId,
        near_amount: Balance,
        attached_deposit: Balance,
        refundable_near_amount: Balance,
        token_amount: Balance,
        storage_staking_price_per_additional_accounts: Balance
    ) -> bool {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let mut validator = match self.validating.validator_registry.get(&validator_account_id) {
                    Some(validator_) => validator_,
                    None => {
                        env::panic_str("Nonexecutable code. Object must exist.");
                    }
                };
                validator.investment_staked_balance += near_amount;
                self.validating.validator_registry.insert(&validator_account_id, &validator);

                let mut investor_investment = match self.validating.investor_investment_registry.get(&predecessor_account_id) {
                    Some(investor_investment_) => investor_investment_,
                    None => {
                        env::panic_str("Nonexecutable code. Object must exist.");
                    }
                };
                let mut staked_balance = match investor_investment.distribution_registry.get(&validator_account_id) {
                    Some(staked_balance_) => staked_balance_,
                    None => {
                        investor_investment.distributions_quantity += 1;

                        0
                    }
                };
                staked_balance += near_amount;
                investor_investment.distribution_registry.insert(&validator_account_id, &staked_balance);
                investor_investment.staked_balance += near_amount;
                self.validating.investor_investment_registry.insert(&predecessor_account_id, &investor_investment);

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

                self.fund.investment_staked_balance += near_amount;

                if refundable_near_amount > 0 {
                    Promise::new(predecessor_account_id.clone())
                        .transfer(refundable_near_amount);
                }

                let current_account_id = env::current_account_id();
                env::log_str(
                    format!(
                        "
                        Deposited to @{} via @{} in {} epoch.
                        Attached deposit is {} yoctoNear.
                        Exchangeable deposit is {} yoctoNear.
                        Reserved storage staking price is {} yoctoNear.
                        Refundable deposit is {} yoctoNear.
                        Old @{} total supply is {} yoctoStNear.
                        Old @{} balance is {} yoctoNear.
                        Old @{} balance is {} yoctoStNear.
                        @{} received {} yoctoStNear.
                        New @{} balance is {} yoctoStNear.
                        New @{} balance is {} yoctoNear.
                        New @{} total supply is {} yoctoStNear.
                        ",
                        &validator_account_id,
                        &current_account_id,
                        env::epoch_height(),
                        attached_deposit,
                        near_amount,
                        storage_staking_price_per_additional_accounts,
                        refundable_near_amount,
                        &current_account_id,
                        self.fungible_token.total_supply - token_amount,
                        &current_account_id,
                        self.fund.get_common_balance() - near_amount,
                        &predecessor_account_id,
                        token_balance - token_amount,
                        &predecessor_account_id,
                        token_amount,
                        &predecessor_account_id,
                        token_balance,
                        &current_account_id,
                        self.fund.get_common_balance(),
                        &current_account_id,
                        self.fungible_token.total_supply
                    ).as_str()
                );

                true
            }
            _ => {
                Promise::new(predecessor_account_id)
                    .transfer(attached_deposit);

                false
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
                self.fund.classic_unstaked_balance -= near_amount;
                self.fund.classic_staked_balance += near_amount;

                let mut validator = match self.validating.validator_registry.get(&validator_account_id) {
                    Some(validator_) => validator_,
                    None => {
                        env::panic_str("Nonexecutable code. Object must exist.");
                    }
                };
                validator.classic_staked_balance += near_amount;
                validator.last_classic_stake_increasing_epoch_height = Some(current_epoch_height);
                self.validating.validator_registry.insert(&validator_account_id, &validator);

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
    ) -> CallbackResult {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let mut validator = match self.validating.validator_registry.get(&validator_account_id) {
                    Some(validator_) => validator_,
                    None => {
                        env::panic_str("Nonexecutable code. Object must exist.");
                    }
                };

                match stake_decreasing_type {
                    StakeDecreasingType::Classic => {
                        validator.classic_staked_balance -= near_amount;
                        self.fund.delayed_withdrawn_fund.needed_to_request_classic_near_amount -= near_amount;
                    }
                    StakeDecreasingType::Investment => {
                        let mut investment_withdrawal = match self.fund.delayed_withdrawn_fund.investment_withdrawal_registry.get(&validator_account_id) {
                            Some(investment_withdrawal_) => investment_withdrawal_,
                            None => {
                                env::panic_str("Nonexecutable code. Object must exist.");
                            }
                        };
                        if near_amount < investment_withdrawal.near_amount {
                            investment_withdrawal.near_amount -= near_amount;

                            self.fund.delayed_withdrawn_fund.investment_withdrawal_registry.insert(&validator_account_id, &investment_withdrawal);
                        } else {
                            self.fund.delayed_withdrawn_fund.investment_withdrawal_registry.remove(&validator_account_id);

                            Promise::new(investment_withdrawal.account_id)
                                .transfer(refundable_near_amount);
                        }

                        validator.investment_staked_balance -= near_amount;
                        self.fund.delayed_withdrawn_fund.needed_to_request_investment_near_amount -= near_amount;
                    }
                }

                validator.unstaked_balance += near_amount;
                self.validating.validator_registry.insert(&validator_account_id, &validator);

                CallbackResult {
                    is_success: true,
                    network_epoch_height: env::epoch_height()
                }
            }
            _ => {
                CallbackResult {
                    is_success: false,
                    network_epoch_height: env::epoch_height()
                }
            }
        }
    }

    #[private]
    pub fn take_unstaked_balance_callback(&mut self, validator_account_id: AccountId) -> CallbackResult {  // TODO Может быть, ставить счетчик на количество валиаторов, с которыз нужно снимать стейк, чтобы проверять.
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let mut validator = match self.validating.validator_registry.get(&validator_account_id) {
                    Some(validator_) => validator_,
                    None => {
                        env::panic_str("Nonexecutable code. Object must exist.");
                    }
                };

                self.fund.delayed_withdrawn_fund.balance += validator.unstaked_balance;

                validator.unstaked_balance = 0;
                self.validating.validator_registry.insert(&validator_account_id, &validator);

                CallbackResult {
                    is_success: true,
                    network_epoch_height: env::epoch_height()
                }
            }
            _ => {
                CallbackResult {
                    is_success: false,
                    network_epoch_height: env::epoch_height()
                }
            }
        }
    }

    // TODO комментарий написать. Возвращаем и сохраняем епохи в разном состоянии по-разному, чтобы запомнить, что в какой эпохе инициировано по фактту, а в какую выполнен коллбек
    #[private]
    pub fn update_validator_callback(
        &mut self,
        validator_account_id: AccountId,
        current_epoch_height: EpochHeight
    ) -> CallbackResult {
        if env::promise_results_count() == 0 {
            env::panic_str("Contract expected a result on the callback.");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(data) => {
                let new_staked_balance: u128 = match near_sdk::serde_json::from_slice::<U128>(data.as_slice()) {
                    Ok(new_staked_balance_) => new_staked_balance_.into(),
                    Err(_) => {
                        env::panic_str("Nonexecutable code. It should be valid JSON object.");
                    }
                };

                let mut validator = match self.validating.validator_registry.get(&validator_account_id) {
                    Some(validator_) => validator_,
                    None => {
                        env::panic_str("Nonexecutable code. Object must exist.");
                    }
                };

                let staking_rewards_near_amount = new_staked_balance - validator.get_staked_balance();

                validator.last_update_epoch_height = current_epoch_height;
                validator.classic_staked_balance += staking_rewards_near_amount;

                self.validating.validator_registry.insert(&validator_account_id, &validator);
                self.validating.quantity_of_validators_updated_in_current_epoch += 1;

                self.previous_epoch_rewards_from_validators_near_amount += staking_rewards_near_amount;

                CallbackResult {
                    is_success: true,
                    network_epoch_height: env::epoch_height()
                }
            }
            _ => {
                CallbackResult {
                    is_success: false,
                    network_epoch_height: env::epoch_height()
                }
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




// TODO TODO TODO TODO TODO ВСе коллбеки сделать так, чтоы приходило БорщСериалайзед данные, а не В Джсоне


//  TODO TODO TODO ВСе Итераторы на Вью делать через индекс.     https://github.com/NearDeFi/burrowland/blob/0dbfa1803bf26353ffbee2ffd4f494bab23b2756/contract/src/account.rs#L207

// TODO TODO TODO TODO TODO Важно запрашивать необходимое количество газа, чтобы хватило на  контракт + кроссколл + коллбек. Иначе что-то выполнится, а что-то нет.

// TODO C Валидатора, по идее, придет немного больше неар, чем запрошено по методам, так как мы ожидаем запрос на Анстейк 4 эпохи, в это время количество отдаваемого зафиксировано, но оно еще приносит прибыль (затем еще 4 эпохи, чтобы забрать), что с этим делать?

// TODO В каждом методе проверить, что все, что взято из хранилища, в него и положено. (get, insert.) .
// TODO проверить, что взятые из хранилища параметры изменяются там, где это требуется.

// TODO проверить, нет ли такого, чтобы пользователб мог что-то сделать за другого пользователя. То есть. АккаунтАйди передается в сигнатуру, а не берется ПредецессорАккаунтАйди


// TODO TODO TODO TODO TODO Можно ли будет перейти на МУЛЬТИСИГ флоу управления после деплоя классического флоу управления.

// TODO логировать

// проверить, что у каждого свойства структуры есть инкремент и дикремент.


// https://nomicon.io/Standards  Евенты и подобное.



// // use near_contract_standards::fungible_token::core::FungibleTokenCore;
// use near_contract_standards::fungible_token::events  - посмотреть, какие нужны
// use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
// https://learn.figment.io/tutorials/stake-fungible-token

// Нужна ли фабрика?

// TODO проработать логи, то есть, втсавлять в них конкретные значения

// TODO подумать, как поступать с аккаунтами при "снятии в ноль". Сейчас аккаунт удаляется и средства за сс отдаются, но можно и просто оставлять аккаунт с 0 значенями

// НАписать DecreaseValidatorStake относительно менеджера, причем это не должно влиять на возможность снятия средств пользователями.


// Проверить, что Методы для кросскола, правильно интегрированы в логику, то есть, где get_account_total_balance/get_account_staked_balance. Проверить все методы.!!!!!!!1

// Документация

// НУжно ли добавить метод рестейк внутрь пула для каждого валидатора?

// сделать логи, и запустиь на крон. Затем сделать курс через добавочный фонд


// Анстейкед Баланс на структуре валидатора и контракте не сходится. Проверить на новом контракте, почему так.



// pool30 задеплоить.


// измененяи михаила