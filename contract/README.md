# Stake pool contract

This contract provides a way for other users to delegate funds to pool of validation nodes.

There are some different roles:
- The staking pool contract. An account with the contract that staking pools funds.
- The staking pool is owned by the `owner` and the `owner` is the general manager of the staking pool.
- The pool `manager` manages the pool. The `manager` is assigned by the `owner` of the staking pool and can be changed anytime.
- Delegator accounts - accounts that want to stake their funds with the staking pool.
- Delegator accounts can become an `investor`.

## Reward distribution

The reward is distributed by increasing the exchange rate of the pool's native token at the begining of each epoch.

Every epoch validators bring rewards to the pool. So, at the beginning of each epoch, the pool synchronizes and updates the information about the native tokens under management from all validators and calculates a new exchange rate for the native token.

## Stake pool contract guarantees and invariants

This staking pool implementation guarantees the required properties of the staking pool standard:

- The contract can't lose or lock tokens of users.
- If a user deposited X, the user should be able to withdraw at least X.
- If a user successfully staked X, the user can unstake at least X.
- The contract should not lock unstaked funds for longer than 8 epochs after delayed withdraw action.

It also has inner invariants:

- The price of staking pool tokens is always at least `1`.
- The price of staking pool tokens never decreases.
- The comission is a fraction be from `0` to `1` inclusive.
- The owner can't withdraw funds from other delegators.

## Implementation details

The owner can setup such contract with different parameters and start receiving users native tokens.
Any other user can send their native tokens to the contract and increase the total stake distributed on validators and receive staking pool fungible tokens.
These users are rewarded by increasing the rate of the staking pool tokens they received, but the contract has the right to charge a commission.
Then users can withdraw their native tokens after some unlocking period by exchanging staking pool tokens.

The price of a staking pool token defined as the total amount of staked native tokens divided by the the total amount of staking pool token.
The number of staking pool token is always less than the number of the staked native tokens, so the price of single staking pool token is not less than `1`.

## Existing `call` methods:
- `new`

Available for pool owner.

Initializes staking pool state.

```rust
#[init]
pub fn new(
        fungible_token_metadata: FungibleTokenMetadataDto,
        manager_id: Option<AccountId>,
        self_fee_receiver_account_id: AccountId,
        partner_fee_receiver_account_id: AccountId,
        reward_fee_self: Option<Fee>,
        reward_fee_partner: Option<Fee>,
        instant_withdraw_fee_self: Option<Fee>,
        instant_withdraw_fee_partner: Option<Fee>
    ) -> Self
```

near deploy --wasmFile ./target/wasm32-unknown-unknown/release/stake_pool.wasm --accountId=pool.testnet --initDeposit=1 --initArgs='{"fungible_token_metadata": {"name": "NAME", "symbol": "SYMBOL", "icon": "ICON", "reference": null, "reference_hash": null, "decimals": 24}, "manager_id": "account0.testnet", "self_fee_receiver_account_id": "account1.testnet", "partner_fee_receiver_account_id": "account2.testnet", "reward_fee_self": {"numerator": 1, "denominator": 100}, "reward_fee_partner": {"numerator": 1, "denominator": 100}, "instant_withdraw_fee_self": {"numerator": 3, "denominator":1000}, "instant_withdraw_fee_partner": {"numerator": 1, "denominator": 5}}'

- `deposit`

Available for all users.

The delegator makes a deposit of funds, and receiving pool tokens in return.
When a delegator account first deposits funds to the contract, the internal account is created and credited with the
`near_amount` native tokens. The attached deposit must be greater than `near_amount` to hide the storage staking,
with the excess fund being refunded.

```rust
#[payable]
pub fn deposit(&mut self, near_amount: U128) -> PromiseOrValue<()>
```
near call pool.testnet deposit '{"near_amount": "10000000000000000000000000"}' --deposit=2 --accountId=account3.testnet --gas=300000000000000

- `deposit_on_validator`

Available for investors.

The delegator makes a deposit of funds via pool directly on validator, and receiving pool tokens in return.
When a delegator account first deposits funds to the contract, the internal account is created and credited with the
`near_amount` native tokens. The attached deposit must be greater than `near_amount` to hide the storage staking,
with the excess fund being refunded.

```rust
#[payable]
pub fn deposit_on_validator(&mut self, near_amount: U128, validator_account_id: AccountId) -> Promise
```
near call pool.testnet deposit_on_validator '{"near_amount": "1000000000000000000000000", "validator_account_id": "legends.pool.f863973.m0"}' --accountId=account3.testnet --deposit=2 --gas=300000000000000

- `instant_withdraw`

Available for all users.

The delegator makes an instant unstake by exchanging the pool tokens he has for native tokens. Native tokens are returned
to the delegator immediately, so there may be a commission for this action.

```rust
#[payable]
pub fn instant_withdraw(&mut self, token_amount: U128) -> Promise
```
near call pool.testnet instant_withdraw '{"token_amount":"1000000000000000000000000"}' --accountId=account3.testnet --deposit=1 --gas=300000000000000

- `delayed_withdraw`

Available for all users.

The delegator makes an unstake by exchanging the pool tokens he has for native tokens. Native tokens can be returned
to the delegator only after 8 epochs.

```rust
#[payable]
pub fn delayed_withdraw(&mut self, token_amount: U128) -> PromiseOrValue<()>
```
near call pool.testnet delayed_withdraw '{"token_amount": "1000000000000000000000000"}' --accountId=account3.testnet --deposit=1 --gas=300000000000000

- `delayed_withdraw_from_validator`

Available for investors.

The delegator makes an unstake via pool directly from validator by exchanging the pool tokens he has for native tokens. Native tokens can be returned
to the delegator only after 8 epochs.

```rust
#[payable]
pub fn delayed_withdraw_from_validator(&mut self, near_amount: U128, validator_account_id: AccountId) -> PromiseOrValue<()>
```
near call pool.testnet delayed_withdraw_from_validator '{"near_amount": "1000000000000000000000000", "validator_account_id": "legends.pool.f863973.m0"}' --accountId=account3.testnet --deposit=1 --gas=300000000000000

- `take_delayed_withdrawal`

Available for all users.

The delegator takes Native tokens he requested after passing the delayed unstake process.

```rust
#[payable]
pub fn take_delayed_withdrawal(&mut self) -> Promise
```
near call pool.testnet take_delayed_withdrawal --accountId=account3.testnet --deposit=1 --gas=300000000000000

- `increase_validator_stake`

Available for pool manager.

Stakes unstaked funds to the validator.

```rust
pub fn increase_validator_stake(&mut self, validator_account_id: AccountId, near_amount: U128) -> Promise
```
near call pool.testnet increase_validator_stake '{"validator_account_id":"legends.pool.f863973.m0", "near_amount":"1000000000000000000000000"}' --accountId=account0.testnet --gas=300000000000000

- `requested_decrease_validator_stake`

Available for pool manager.

Unstakes staked funds from validator.

```rust
pub fn requested_decrease_validator_stake(
    &mut self,
    validator_account_id: AccountId,
    near_amount: U128,
    stake_decreasing_type: StakeDecreasingType
) -> Promise
```
near call pool.testnet requested_decrease_validator_stake '{"validator_account_id":"legends.pool.f863973.m0", "near_amount":"500000000000000000000000", "stake_decreasing_type":"Classic"}' --accountId=account0.testnet --gas=300000000000000

- `take_unstaked_balance`

Available for pool manager.

Takes requested to withdraw balance from validator.

```rust
pub fn take_unstaked_balance(&mut self, validator_account_id: AccountId) -> Promise
```
near call pool.testnet take_unstaked_balance '{"validator_account_id":"legends.pool.f863973.m0"}' --accountId=account0.testnet --gas=300000000000000

- `update_validator`

Available for pool manager.

Updates validator state.

```rust
pub fn update_validator(&mut self, validator_account_id: AccountId) -> Promise
```
near call pool.testnet update_validator '{"validator_account_id":"legends.pool.f863973.m0"}' --accountId=account0.testnet --gas=300000000000000

- `update`

Available for pool manager.

Updates pool state.

```rust
pub fn update(&mut self)
```
near call pool.testnet update --accountId=account0.testnet --gas=300000000000000

- `add_validator`

Available for pool manager.

Adds the validator to the list of validators to which the pool delegates the available native tokens.

```rust
pub fn add_validator(
    &mut self,
    validator_account_id: AccountId,
    staking_contract_version: StakingContractVersion,
    is_only_for_investment: bool,
    is_preferred: bool
) -> PromiseOrValue<()>
```
near call pool.testnet add_validator '{"validator_account_id":"legends.pool.f863973.m0","staking_contract_version":"Core", "is_only_for_investment": false, "is_preferred": true}' --accountId=account0.testnet --deposit=1 --gas=300000000000000

- `change_validator_investment_context`

Available for pool manager.

Changes validator state in contex of investment flow.

```rust
pub fn change_validator_investment_context(&mut self, validator_account_id: AccountId, is_only_for_investment: bool)
```
near call pool.testnet change_validator_investment_context '{"validator_account_id":"legends.pool.f863973.m0", "is_only_for_investment": false}' --accountId=account0.testnet --gas=300000000000000

- `change_preffered_validator`

Available for pool manager.

Changes preffered validator.

```rust
pub fn change_preffered_validator(&mut self, validator_account_id: Option<AccountId>)
```
near call pool.testnet change_preffered_validator '{"validator_account_id":"legends.pool.f863973.m0", "is_only_for_investment": false}' --accountId=account0.testnet --gas=300000000000000

- `remove_validator`

Available for pool manager.

Removes the validator from the list of validators to which the pool delegates the available native tokens.

```rust
pub fn remove_validator(&mut self, validator_account_id: AccountId) -> Promise
```
near call pool.testnet remove_validator '{"validator_account_id":"legends.pool.f863973.m0"}' --accountId=account0.testnet --gas=300000000000000

- `add_investor`

Available for pool manager.

Adds the user to the list of investors.

```rust
#[payable]
pub fn add_investor(&mut self, investor_account_id: AccountId) -> PromiseOrValue<()>
```
near call pool.testnet add_investor '{"investor_account_id":"account4.testnet"}' --accountId=account0.testnet --deposit=1 --gas=300000000000000

- `remove_investor`

Available for pool manager.

Remove the user from the list of investors.

```rust
pub fn remove_investor(&mut self, investor_account_id: AccountId) -> Promise
```
near call pool.testnet remove_investor '{"investor_account_id":"account4.testnet"}' --accountId=account0.testnet --deposit=1 --gas=300000000000000

- `change_manager`

Available for pool owner and manager.

Changes pool manager.

```rust
pub fn change_manager(&mut self, manager_id: AccountId)
```
near call pool.testnet change_manager '{"manager_id":"account5.testnet"}' --accountId=account0.testnet --gas=300000000000000

- `change_reward_fee`

Available for pool manager.

Changes fee for validators rewards.

```rust
pub fn change_reward_fee(&mut self, reward_fee_self: Option<Fee>, reward_fee_partner: Option<Fee>)
```
near call pool.testnet change_reward_fee '{"reward_fee_self": {"numerator": 1, "denominator": 100}, "reward_fee_partner": {"numerator": 1, "denominator": 100}}' --accountId=account0.testnet --gas=300000000000000

- `change_instant_withdraw_fee`

Available for pool manager.

Changes fee for instant unstake process.

```rust
pub fn change_instant_withdraw_fee(&mut self, instant_withdraw_fee_self: Option<Fee>, instant_withdraw_fee_partner: Option<Fee>)
```
near call pool.testnet change_instant_withdraw_fee '{"instant_withdraw_fee_self": {"numerator": 1, "denominator": 100}, "instant_withdraw_fee_partner": {"numerator": 1, "denominator": 100}}' --accountId=account0.testnet --gas=300000000000000

- `confirm_stake_distribution`

Available for pool manager.

Confirms stake distributions.

```rust
pub fn confirm_stake_distribution(&mut self)
```
near call pool.testnet confirm_stake_distribution --accountId=account0.testnet --gas=300000000000000

## Existing `view` methods:
```rust
pub fn get_delayed_withdrawal_details(&self, account_id: AccountId) -> Option<DelayedWithdrawalDetails>
```
near view pool.testnet get_delayed_withdrawal_details '{"account_id": "account6.testnet"}'


```rust
pub fn get_account_balance(&self, account_id: AccountId) -> AccountBalanceDto
```
near view pool.testnet get_account_balance '{"account_id": "account6.testnet"}'


```rust
pub fn get_total_token_supply(&self) -> U128
```
near view pool.testnet get_total_token_supply


```rust
pub fn get_minimum_deposit_amount(&self) -> U128
```
near view pool.testnet get_minimum_deposit_amount


```rust
pub fn get_storage_staking_price(&self) -> StorageStakingPrice
```
near view pool.testnet get_storage_staking_price


```rust
pub fn get_storage_staking_requested_coverage(&self, account_id: AccountId) -> StorageStakingRequestedCoverage
```
near view pool.testnet get_storage_staking_requested_coverage '{"account_id": "account6.testnet"}'


```rust
pub fn get_fund(&self) -> FundDto
```
near view pool.testnet get_fund

```rust
pub fn get_fee_registry(&self) -> FeeRegistry
```
near view pool.testnet get_fee_registry

```rust
pub fn get_fee_registry_light(&self) -> FeeRegistryLight
```
near view pool.testnet get_fee_registry_light

```rust
pub fn get_current_epoch_height(&self) -> EpochHeightRegistry
```
near view pool.testnet get_current_epoch_height

```rust
pub fn is_stake_distributed(&self) -> bool
```
near view pool.testent is_stake_distributed

```rust
pub fn get_investor_investment(&self, account_id: AccountId) -> Option<InvestorInvestmentDto>
```
near view pool.testnet get_investor_investment '{"account_id": "account6.testnet"}'

```rust
pub fn get_validator_registry(&self) -> Vec<ValidatorDto>
```
near view pool.testnet get_validator_registry

```rust
pub fn get_preffered_validator(&self) -> Option<ValidatorDto>
```
near view pool.testnet get_preffered_validator

```rust
pub fn get_aggregated(&self) -> Aggregated
```
near view pool.testnet get_aggregated

```rust
pub fn get_requested_to_withdrawal_fund(&self) -> RequestedToWithdrawalFund
```
near view pool.testnet get_requested_to_withdrawal_fund

```rust
pub fn get_full(&self) -> Full
```
near view pool.testnet get_full

```rust
pub fn get_full_for_account(&self, account_id: AccountId) -> FullForAccount
```
near view pool.testnet get_full for account '{"account_id": "account6.testnet"}'