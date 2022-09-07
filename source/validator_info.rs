use near_sdk::{env, EpochHeight, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::validator_staking_contract_version::ValidatorStakingContractVersion;
use super::delayed_unstake_validator_group::DelayedUnstakeValidatorGroup;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorInfo {
    pub delayed_unstake_validator_group: DelayedUnstakeValidatorGroup,
    pub staking_contract_version: ValidatorStakingContractVersion,
    pub staked_balance: Balance,
    pub last_update_info_epoch_height: EpochHeight,
    pub last_stake_increasing_epoch_height: Option<EpochHeight>
}

impl ValidatorInfo {
    pub fn new(
        validator_staking_contract_version: ValidatorStakingContractVersion,
        delayed_unstake_validator_group: DelayedUnstakeValidatorGroup
    ) -> Self {
        Self {
            delayed_unstake_validator_group,
            staking_contract_version: validator_staking_contract_version,
            staked_balance: 0,
            last_update_info_epoch_height: env::epoch_height(),
            last_stake_increasing_epoch_height: None
        }
    }
}