use near_sdk::{env, EpochHeight, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::validator_staking_contract_version::ValidatorStakingContractVersion;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorInfo {
    pub staking_contract_version: ValidatorStakingContractVersion,
    pub staked_balance: Balance,
    pub unstaked_balance: Balance, // TODO МОжет быть, для подсчета сторэжСтейкинг нужно класть не None. Как механизм подсчета считает занятое пространстов. То есть, Выделено на Option<U8> - 2 байта, а занято для None - 1 или 2. По идее, все место переменной занято, .
    pub last_update_info_epoch_height: EpochHeight,
    pub last_stake_increasing_epoch_height: Option<EpochHeight>
}

impl ValidatorInfo {
    pub fn new(validator_staking_contract_version: ValidatorStakingContractVersion) -> Self {
        Self {
            staking_contract_version: validator_staking_contract_version,
            staked_balance: 0,
            unstaked_balance: 0,
            last_update_info_epoch_height: env::epoch_height(),
            last_stake_increasing_epoch_height: None
        }
    }
}