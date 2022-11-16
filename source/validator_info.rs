use near_sdk::{env, EpochHeight, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::validator_staking_contract_version::ValidatorStakingContractVersion;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorInfo {
    pub classic_staked_balance: Balance,
    pub investment_staked_balance: Balance,
    pub unstaked_balance: Balance,
    pub staking_contract_version: ValidatorStakingContractVersion,
    /// Validator, which is needed ONLY for investment purpose.
    /// The pool should not distribute unstaked balance to validators with a TRUE value,
    /// and this means, that classic staked balance must always be equal to zero and investment
    /// staked balance can be greater than zero. The pool should distribute unstaked balance
    /// only to validators with a FALSE value, and it is also possible to use the validator for
    /// investment purposes, this means, that classic staked balance and investment staked balance
    /// can be greater than zero.
    pub is_only_for_investment: bool,
    pub last_update_info_epoch_height: EpochHeight,     // TODO поменять название
    pub last_classic_stake_increasing_epoch_height: Option<EpochHeight>
}

impl ValidatorInfo {
    pub fn new(
        validator_staking_contract_version: ValidatorStakingContractVersion,
        is_only_for_investment: bool
    ) -> Self {
        Self {
            classic_staked_balance: 0,
            investment_staked_balance: 0,
            unstaked_balance: 0,
            staking_contract_version: validator_staking_contract_version,
            is_only_for_investment,
            last_update_info_epoch_height: env::epoch_height(),
            last_classic_stake_increasing_epoch_height: None
        }
    }
}