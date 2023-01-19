use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::delayed_withdrawn_fund::DelayedWithdrawnFund;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Fund {
    /// Near amount available in the pool that should be staked on validators in classic context.
    pub classic_unstaked_balance: Balance,
    /// Near amount already staked on validators in classic context.
    pub classic_staked_balance: Balance,
    /// Additional Near amount to ensure the possibility of instant withdrawal.
    pub classic_liquidity_balance: Balance,
    /// Near amount already staked on validators in investment context.
    pub investment_staked_balance: Balance,
    /// Fund that should be returned to users.
    pub delayed_withdrawn_fund: DelayedWithdrawnFund,
    pub is_distributed_on_validators_in_current_epoch: bool
}

impl Fund {
    pub fn new() -> Self {
        Self {
            classic_unstaked_balance: 0,
            classic_staked_balance: 0,
            classic_liquidity_balance: 0,
            investment_staked_balance: 0,
            delayed_withdrawn_fund: DelayedWithdrawnFund::new(),
            is_distributed_on_validators_in_current_epoch: false
        }
    }

    pub fn get_staked_balance(&self) -> Balance {
        self.classic_staked_balance + self.investment_staked_balance
    }

    pub fn get_common_balance(&self) -> Balance {
        self.classic_unstaked_balance + self.classic_staked_balance + self.investment_staked_balance
    }
}