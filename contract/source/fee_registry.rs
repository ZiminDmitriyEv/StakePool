use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::clone::Clone;
use super::shared_fee::SharedFee;

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FeeRegistry {
    pub reward_fee: Option<SharedFee>,
    pub instant_withdraw_fee: Option<SharedFee>
}