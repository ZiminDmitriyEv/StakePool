use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::base_error::BaseError;
use near_sdk::{env, EpochHeight};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorInfo {
    staked_balance: Balance,            // TODO добавить ContractVersion ВСе контракты разные. ??????????
    epoch_height_for_last_update: EpochHeight
    // TODO stake shares
}

impl ValidatorInfo {
    pub fn new() -> Self {
        Self {
            staked_balance: 0,       // Todo как узнать, сколько денег на каждом валидаотер
            epoch_height_for_last_update: env::epoch_height()
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