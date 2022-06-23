use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::clone::Clone;

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Fee {
    numerator: u64,
    denominator: u64
}

impl Fee {
    pub fn is_valid(&self) -> bool {
        return self.denominator != 0 && self.numerator != 0 && self.numerator < self.denominator;
    }
}