use near_sdk::{env, EpochHeight, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::base_error::BaseError;
use super::validator_staking_contract_version::ValidatorStakingContractVersion;
use super::delayed_unstake_validator_group::DelayedUnstakeValidatorGroup;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorInfo {
    delayed_unstake_validator_group: DelayedUnstakeValidatorGroup,
    staking_contract_version: ValidatorStakingContractVersion,
    staked_balance: Balance,
    last_update_info_epoch_height: EpochHeight,
    last_stake_increasing_epoch_height: Option<EpochHeight>
}

impl ValidatorInfo {
    pub fn new(
        validator_staking_contract_version: ValidatorStakingContractVersion, delayed_unstake_validator_group: DelayedUnstakeValidatorGroup
    ) -> Self {
        Self {
            delayed_unstake_validator_group,
            staking_contract_version: validator_staking_contract_version,
            staked_balance: 0,
            last_update_info_epoch_height: env::epoch_height(),
            last_stake_increasing_epoch_height: None
        }
    }

    pub fn increase_staked_balance(&mut self, yocto_near_amount: Balance) -> Result<(), BaseError> {
        self.staked_balance = match self.staked_balance
            .checked_add(yocto_near_amount) {
            Some(staked_balance) => {
                staked_balance
            }
            None => {
                return Err(BaseError::CalculationOwerflow);
            }
        };

        Ok(())
    }

    pub fn set_staked_balance(&mut self, yocto_near_amount: Balance) {
        self.staked_balance = yocto_near_amount;
    }

    pub fn set_last_update_info_epoch_height(&mut self, last_update_info_epoch_height: EpochHeight) {
        self.last_update_info_epoch_height = last_update_info_epoch_height;
    }

    pub fn set_last_stake_increasing_epoch_height(&mut self, last_stake_increasing_epoch_height: EpochHeight) {
        self.last_stake_increasing_epoch_height = Some(last_stake_increasing_epoch_height);
    }

    pub fn get_staking_contract_version(&self) -> &ValidatorStakingContractVersion {
        &self.staking_contract_version
    }

    pub fn get_staked_balance(&self) -> Balance {
        self.staked_balance
    }

    pub fn get_last_update_info_epoch_haight(&self) -> EpochHeight {
        self.last_update_info_epoch_height
    }
}