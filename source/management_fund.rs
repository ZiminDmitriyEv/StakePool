use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::base_error::BaseError;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ManagementFund {
    available_for_staking_balance: Balance,
    staked_balance: Balance
}

impl ManagementFund {
    pub fn new() -> Self {
        Self {
            available_for_staking_balance: 0,
            staked_balance: 0
        }
    }

    pub fn increase_available_for_staking_balance(&mut self, yocto_near_amount: Balance) -> Result<(), BaseError> {
        self.available_for_staking_balance = match self.available_for_staking_balance
            .checked_add(yocto_near_amount) {
            Some(available_for_staking_balance_) => {
                available_for_staking_balance_
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        };

        Ok(())
    }

    pub fn decrease_available_for_staking_balance(&mut self, yocto_near_amount: Balance) -> Result<(), BaseError> {
        self.available_for_staking_balance = match self.available_for_staking_balance
            .checked_sub(yocto_near_amount) {
            Some(available_for_staking_balance_) => {
                available_for_staking_balance_
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        };

        Ok(())
    }

    pub fn increase_staked_balance(&mut self, yocto_near_amount: Balance) -> Result<(), BaseError> {
        self.staked_balance = match self.staked_balance
            .checked_add(yocto_near_amount) {
            Some(staked_balance_) => {
                staked_balance_
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        };

        Ok(())
    }

    pub fn decrease_staked_balance(&mut self, yocto_near_amount: Balance) -> Result<(), BaseError> {
        self.staked_balance = match self.staked_balance
            .checked_sub(yocto_near_amount) {
            Some(staked_balance_) => {
                staked_balance_
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        };

        Ok(())
    }

    pub fn get_available_for_staking_balance(&self) -> Balance {
        self.available_for_staking_balance
    }

    pub fn get_staked_balance(&self) -> Balance {
        self.staked_balance
    }

    pub fn get_management_fund_amount(&self) -> Result<Balance, BaseError> {
        match self.available_for_staking_balance
            .checked_add(self.staked_balance) {
            Some(management_fund_amount) => {
                Ok(management_fund_amount)
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        }
    }
}