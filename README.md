# Stake pool contract

This contract provides a way for other users to delegate funds to pool of validation nodes.

There are some different roles:
- The staking pool contract. An account with the contract that staking pools funds.
- The staking pool is owned by the `owner` and the `owner` is the general manager of the staking pool.
- The pool `manager` manages the pool. The `manager` is assigned by the `owner` of the staking pool and can be changed anytime.
- Delegator accounts (`user1`, `user2`, etc.) - accounts that want to stake their funds with the staking pool.

## Implementation details

The owner can setup such contract with different parameters and start receiving users native tokens.
Any other user can send their native tokens to the contract and increase the total stake distributed on validators and receive staking pool fungible tokens.
These users are rewarded by increasing the rate of the staking pool tokens they received, but the contract has the right to charge a commission.
Then users can withdraw their native tokens after some unlocking period by exchanging staking pool tokens.

The price of a staking pool token defined as the total amount of staked native tokens divided by the the total amount of staking pool token.
The number of staking pool token is always less than the number of the staked native tokens, so the price of single staking pool token is not less than `1`.

## Initialization

A contract has to be initialized with the following parameters:
- `manager_id` - `string` the account ID of the contract owner. This account will be able to call owner-only methods. E.g. `owner`
- `rewards_fee` - `json serialized object` the initial value of the fraction of the reward that the owner charges delegators for staking pool managment.
- `validators_maximum_quantity` - `integer` - maximum quantity of validators. Can be changed anytime.

During the initialization the contract checks validity of the input and initializes the contract.
The contract shouldn't have locked balance during the initialization.

## Existing methods:
- `new`

`call` method.

Initializes staking pool state.

```rust
#[init]
pub fn new(
        manager_id: Option<AccountId>,
        rewards_fee: Option<Fee>,
        validators_maximum_quantity: Option<u64>
    ) -> Self
```

- `deposit`

Available to the client. `call` method.

The delegator makes a deposit of funds, and receiving pool tokens in return.
When a delegator account first deposits funds to the contract, the internal account is created and credited with the
attached amount of unstaked native tokens.

```rust
#[payable]
pub fn deposit(
    &mut self
)
```

- `instant_withdraw`

Available to the client. `call` method.

The delegator makes an instant unstake by exchanging the pool tokens he has for native tokens. Native tokens are returned
to the delegator immediately, so there may be a commission for this action.

```rust
pub fn instant_withdraw(
    &mut self,
    yocto_token_amount: U128
) -> Promise
```

- `delayed_withdraw`

Available to the client. `call` method.

The delegator makes an unstake by exchanging the pool tokens he has for native tokens. Native tokens can be returned
to the delegator only after 8 epochs.

```rust
#[payable]
pub fn delayed_withdraw(
    &mut self,
    yocto_token_amount: U128
) -> Promise
```

- `add_validator`

Available to the manager. `call` method.

Adds the validator to the list of validators to which the pool delegates the available native tokens.

```rust
#[payable]
pub fn add_validator(
    &mut self,
    account_id: AccountId,
    validator_staking_contract_version: ValidatorStakingContractVersion,
    delayed_withdrawal_validator_group: DelayedWithdrawalValidatorGroup
)
```

- `remove_validator`

Available to the manager. `call` method.

Removes the validator from the list of validators to which the pool delegates the available native tokens.

```rust
pub fn remove_validator(
    &mut self,
    account_id: AccountId
) -> Promise
```

- `increase_validator_stake`

Available to the manager. `call` method.

Distributes native tokens available on the staking pool contract to validator contracts.

```rust
pub fn increase_validator_stake(
    &mut self,
    validator_account_id: AccountId,
    yocto_near_amount: Balance
) -> Promise
```

- `decrease_validator_stake`

Available to the manager. `call` method.

Withdraws native tokens from the validator contract.

```rust
pub fn decrease_validator_stake(
    &mut self,
    validator_account_id: AccountId,
    yocto_near_amount: Balance
) -> Promise
```

- `update_validator_info`

Available to the manager. `call` method.

Updates information about native tokens distributed on validators at the begining of the new epoch.

```rust
pub fn update_validator_info(
    &mut self,
    validator_account_id: AccountId
) -> Promise
```

- `update`

Available to the manager. `call` method.

Updates information about staking pool itself.
This should be done immediately after updating information about all validators by method `update_validator_info`.

```rust
pub fn update(
    &mut self
)
```

- `change_manager`

Available to the owner or manager. `call` method.

Changes staking pool manager.

```rust
pub fn change_manager(
    &mut self,
    manager_id: AccountId
)
```

- `change_rewards_fee`

Available to the manager. `call` method.

Changes pool comission charged from the reward received from validator.

```rust
pub fn change_rewards_fee(
    &mut self,
    rewards_fee: Option<Fee>
)
```

- `is_account_registered`

Available to the client. `view` method.

Checks for token account existing in staking pool.

```rust
pub fn is_account_registered(
    &self,
    account_id: AccountId
) -> bool
```

- `get_total_token_supply`

Available to the client. `view` method.

Checks quantity of total minted staking pool tokens.

```rust
pub fn get_total_token_supply(
    &self
) -> U128
```

- `get_stakers_quantity`

Available to the client. `view` method.

Checks quantity of stakers - users, wich holds staking pool tokens.

```rust
pub fn get_stakers_quantity(
    &self
) -> u64
```

- `get_storage_staking_price_per_additional_token_account`

Available to the client. `view` method.

Checks quantity of native tokens needed to register additional token account.

```rust
pub fn get_storage_staking_price_per_additional_token_account(
    &self
) -> U128
```

- `get_yocto_token_amount_from_yocto_near_amount`

Available to the client. `view` method.

Checks quantity of staking pool tokens wich he can receive from the given number of native tokens.

```rust
pub fn get_yocto_token_amount_from_yocto_near_amount(
    &self,
    yocto_near_amount: U128
) -> U128
```

- `get_yocto_near_amount_from_yocto_token_amount`

Available to the client. `view` method.

Checks quantity of native tokens wich he can receive from the given number of staking pool tokens.

```rust
pub fn get_yocto_near_amount_from_yocto_token_amount(
    &self,
    yocto_token_amount: U128
) -> U128
```

- `get_token_account_balance`

Available to the client. `view` method.

Checks quantity of staking pool tokens wich he holds.

```rust
pub fn get_token_account_balance(
    &self,
    account_id: AccountId
) -> U128
```

- `get_available_for_staking_balance`

Available to the client. `view` method.

Checks quantity of native tokens in staking pool wich is not distributed on validators yet.

```rust
pub fn get_available_for_staking_balance(
    &self
) -> U128
```

- `get_staked_balance`

Available to the client. `view` method.

Checks quantity of native tokens in staking pool wich already distributed on validators.

```rust
pub fn get_staked_balance(
    &self
) -> U128
```

- `get_management_fund_amount`

Available to the client. `view` method.

Checks quantity of native tokens under staking pool management.

```rust
pub fn get_management_fund_amount(
    &self
) -> U128
```

- `get_fee_registry`

Available to the client. `view` method.

Receives information about existing in staking pool comissions.

```rust
pub fn get_fee_registry(
    &self
) -> FeeRegistry
```

- `get_aggregated_information`

Available to the client. `view` method.

Receives information in convenient way.

```rust
pub fn get_aggregated_information(
    &self
) -> AggregatedInformation
```

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

## Some usage example

```bash
near deploy --wasmFile=./target/wasm32-unknown-unknown/release/stake_pool.wasm --accountId=stake.pool.testnet --initArgs='{"manager_id":"manager.testnet", "rewards_fee":{"numerator": _, "denominator":_}, "validators_maximum_quantity":_}'
```

```bash
near call stake.pool.testnet add_validator '{"account_id":"_","validator_staking_contract_version":"_", "delayed_withdrawal_validator_group":"_" }' --accountId=manager.testnet --deposit=1
```

```bash
near call stake.pool.testnet increase_validator_stake '{"validator_account_id":"_", "yocto_near_amount":_}' --accountId=manager.testnet --gas=300000000000000
```

```bash
near call stake.pool.testnet update_validator_info '{"validator_account_id":"_"}' --accountId=manager.testnet --gas=300000000000000
```

```bash
near call stake.pool.testnet deposit --deposit=1 --accountId=_
```

```bash
near call stake.pool.testnet instant_withdraw '{"yocto_token_amount":"_"}' --accountId=_
```

```bash
near view stake.pool.testnet get_token_account_balance '{"account_id":_}'
```