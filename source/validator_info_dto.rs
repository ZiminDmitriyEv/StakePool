use near_sdk::{EpochHeight, AccountId};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use super::delayed_withdrawal_validator_group::DelayedWithdrawalValidatorGroup;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorInfoDto {// TODO Переназвать. ВСе ДТО ВЫНЕСТИ В модуль.
    pub account_id: AccountId,
    pub delayed_withdrawal_validator_group: DelayedWithdrawalValidatorGroup,
    pub staked_balance: U128,
    pub last_update_info_epoch_height: EpochHeight,
    pub last_stake_increasing_epoch_height: Option<EpochHeight>
}