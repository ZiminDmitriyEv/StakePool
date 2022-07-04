use near_sdk::{env, EpochHeight, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::base_error::BaseError;
use super::validator_staking_contract_version::ValidatorStakingContractVersion;
use super::delayed_unstake_validator_group::DelayedUnstakeValidatorGroup;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorInfo {
    delayed_unstake_validator_group: DelayedUnstakeValidatorGroup,
    staked_balance: Balance,            // TODO добавить ContractVersion ВСе контракты разные. ??????????
    last_update_epoch_height: EpochHeight,
    staking_contract_version: ValidatorStakingContractVersion
}

impl ValidatorInfo {
    pub fn new(
        validator_staking_contract_version: ValidatorStakingContractVersion, delayed_unstake_validator_group: DelayedUnstakeValidatorGroup
    ) -> Self {
        Self {
            delayed_unstake_validator_group,
            staked_balance: 0,       // Todo как узнать, сколько денег на каждом валидаотер
            last_update_epoch_height: env::epoch_height(),
            staking_contract_version: validator_staking_contract_version
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