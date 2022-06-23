use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::base_error::BaseError;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorInfo {
    staked_balance: Balance,
}

impl ValidatorInfo {
    pub fn new() -> Self {
        Self {
            staked_balance: 0       // Todo как узнать, сколько денег на каждом валидаотер
        }
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
}