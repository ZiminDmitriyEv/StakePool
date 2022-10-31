use near_sdk::{env, EpochHeight, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::validator_staking_contract_version::ValidatorStakingContractVersion;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorInfo {
    pub staking_contract_version: ValidatorStakingContractVersion,
    pub classic_staked_balance: Balance,
    pub investment_staked_balance: Balance,
    pub unstaked_balance: Balance,
    pub last_update_info_epoch_height: EpochHeight,     // TODO поменять название
    pub last_classic_stake_increasing_epoch_height: Option<EpochHeight>
}

impl ValidatorInfo {
    pub fn new(validator_staking_contract_version: ValidatorStakingContractVersion) -> Self {
        Self {
            staking_contract_version: validator_staking_contract_version,
            classic_staked_balance: 0,
            investment_staked_balance: 0,
            unstaked_balance: 0,
            last_update_info_epoch_height: env::epoch_height(),
            last_classic_stake_increasing_epoch_height: None
        }
    }
}