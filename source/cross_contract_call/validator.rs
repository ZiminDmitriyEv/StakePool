use near_sdk::{ext_contract, AccountId};
use near_sdk::json_types::U128;

/// Default Near `staking pool` contract interface.
#[ext_contract(validator)]
pub trait Validator {
    // #[payable]
    fn deposit(&mut self);

    // #[payable]
    fn deposit_and_stake(&mut self);

    fn withdraw(&mut self, amount: U128);

    fn withdraw_all(&mut self);

    fn stake(&mut self, amount: U128);

    fn unstake(&mut self, amount: U128);

    fn unstake_all(&mut self);

    fn get_account_staked_balance(&self, account_id: AccountId) -> U128;

    fn get_account_unstaked_balance(&self, account_id: AccountId) -> U128;

    fn get_account_total_balance(&self, account_id: AccountId) -> U128;
}