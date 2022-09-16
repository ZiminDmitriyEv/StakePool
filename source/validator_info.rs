use near_sdk::{env, EpochHeight, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::delayed_withdrawal_validator_group::DelayedWithdrawalValidatorGroup;
use super::validator_staking_contract_version::ValidatorStakingContractVersion;
use super::validator_unstake_info::ValidatorUnstakeInfo;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorInfo {
    pub delayed_withdrawal_validator_group: DelayedWithdrawalValidatorGroup,
    pub staking_contract_version: ValidatorStakingContractVersion,
    pub staked_balance: Balance,
    pub validator_unstake_info: Option<ValidatorUnstakeInfo>, // TODO МОжет быть, для подсчета сторэжСтейкинг нужно класть не None. Как механизм подсчета считает занятое пространстов. То есть, Выделено на Option<U8> - 2 байта, а занято для None - 1 или 2. По идее, все место переменной занято, .
    pub last_update_info_epoch_height: EpochHeight,
    pub last_stake_increasing_epoch_height: Option<EpochHeight>
}

impl ValidatorInfo {
    pub fn new(
        validator_staking_contract_version: ValidatorStakingContractVersion,
        delayed_withdrawal_validator_group: DelayedWithdrawalValidatorGroup
    ) -> Self {
        Self {
            delayed_withdrawal_validator_group,
            staking_contract_version: validator_staking_contract_version,
            validator_unstake_info: None,
            staked_balance: 0,
            last_update_info_epoch_height: env::epoch_height(),
            last_stake_increasing_epoch_height: None
        }
    }
}