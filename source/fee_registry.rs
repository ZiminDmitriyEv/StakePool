use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::clone::Clone;
use super::fee::Fee;

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FeeRegistry {
    pub rewards_fee: Option<Fee>,
    pub everstake_rewards_fee: Option<Fee>
}