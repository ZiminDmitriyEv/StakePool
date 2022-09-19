use near_sdk::{EpochHeight, AccountId};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DelayedWithdrawalInfoDto {
    pub account_id: AccountId,
    pub requested_yocto_near_amount: U128,
    pub received_yocto_near_amount: U128,
    pub started_epoch_height: EpochHeight
}