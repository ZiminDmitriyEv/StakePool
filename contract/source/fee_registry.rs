use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::clone::Clone;
use super::shared_fee::SharedFee;

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FeeRegistry {
    /// Fee that is taken from the rewards received on the validators.
    pub reward_fee: Option<SharedFee>,
    /// Fee that is taken from the Near amount on instant unstake process.
    pub instant_withdraw_fee: Option<SharedFee>
}