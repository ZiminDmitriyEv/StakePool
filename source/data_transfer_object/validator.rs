use near_sdk::{EpochHeight, AccountId};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Validator {
    pub account_id: AccountId,
    pub unstaked_balance: U128,
    pub classic_staked_balance: U128,
    pub investment_staked_balance: U128,
    pub is_only_for_investment: bool,
    pub last_update_epoch_height: EpochHeight,
    pub last_classic_stake_increasing_epoch_height: Option<EpochHeight>
}