use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::delayed_withdrawn_fund::DelayedWithdrawnFund;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Fund {
    pub unstaked_balance: Balance,
    pub staked_balance: Balance,
    pub delayed_withdrawn_fund: DelayedWithdrawnFund,
    /// Not used yet.
    /// Additional funds to ensure the possibility of instant withdrawal.
    pub liquidity_balance: Balance,
    pub is_distributed_on_validators_in_current_epoch: bool         // TODO вынести в ВспомогательныеПараметры такой и подобные параметры или просто переназвать
}

impl Fund {
    pub fn new() -> Self {
        Self {
            unstaked_balance: 0,
            staked_balance: 0,
            delayed_withdrawn_fund: DelayedWithdrawnFund::new(),
            liquidity_balance: 0,
            is_distributed_on_validators_in_current_epoch: false
        }
    }

    pub fn get_fund_amount(&self) -> Balance {
        self.unstaked_balance + self.staked_balance
    }
}