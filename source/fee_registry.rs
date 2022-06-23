use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::clone::Clone;
use super::fee::Fee;

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FeeRegistry {
    rewards_fee: Option<Fee>
}

impl FeeRegistry {
    pub fn new(rewards_fee: Option<Fee>) -> Self {
        Self {
            rewards_fee
        }
    }

    pub fn change_rewards_fee(&mut self, rewards_fee: Option<Fee>) {
        self.rewards_fee = rewards_fee;
    }

    pub fn get_rewards_fee(&self) -> &Option<Fee> {
        &self.rewards_fee
    }
}