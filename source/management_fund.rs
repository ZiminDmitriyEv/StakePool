use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::delayed_withdrawn_fund::DelayedWithdrawnFund;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ManagementFund {
    pub unstaked_balance: Balance,
    pub staked_balance: Balance,
    pub delayed_withdrawn_fund: DelayedWithdrawnFund,
    pub is_distributed_on_validators_in_current_epoch: bool         // TODO вынести в ВспомогательныеПараметры такой и подобные параметры.
}

impl ManagementFund {
    pub fn new() -> Self {
        Self {
            unstaked_balance: 0,
            staked_balance: 0,
            delayed_withdrawn_fund: DelayedWithdrawnFund::new(),
            is_distributed_on_validators_in_current_epoch: false
        }
    }

    pub fn get_management_fund_amount(&self) -> Balance {
        self.unstaked_balance + self.staked_balance
    }
}