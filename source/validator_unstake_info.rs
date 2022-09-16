use near_sdk::{env, EpochHeight, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorUnstakeInfo {
    pub yocto_near_amount: Balance,
    pub epoch_to_withdraw: EpochHeight
}