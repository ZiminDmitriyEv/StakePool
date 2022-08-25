use near_sdk::{EpochHeight, Balance, AccountId};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorInfoDto {// TODO Переназвать. ВСе ДТО ВЫНЕСТИ В модуль.
    account_id: AccountId,
    staked_balance: U128,
    last_update_info_epoch_height: EpochHeight,
    last_stake_increasing_epoch_height: Option<EpochHeight>
}

impl ValidatorInfoDto {
    pub fn new(
        account_id: AccountId,
        staked_balance: Balance,
        last_update_info_epoch_height: EpochHeight,
        last_stake_increasing_epoch_height: Option<EpochHeight>
    ) -> Self {
        Self {
            account_id,
            staked_balance: staked_balance.into(),
            last_update_info_epoch_height,
            last_stake_increasing_epoch_height
        }
    }
}