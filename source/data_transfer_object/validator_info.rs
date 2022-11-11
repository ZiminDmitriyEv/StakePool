use near_sdk::{EpochHeight, AccountId};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorInfo {
    pub account_id: AccountId,
    pub classic_staked_balance: U128,
    pub investment_staked_balance: U128,
    pub last_update_info_epoch_height: EpochHeight,
    pub last_stake_increasing_epoch_height: Option<EpochHeight>
}