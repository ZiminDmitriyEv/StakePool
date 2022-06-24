use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::clone::Clone;
use super::base_error::BaseError;

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Fee {
    numerator: u64,
    denominator: u64
}

impl Fee {
    pub fn assert_valid(&self) -> Result<(), BaseError> {
        if self.denominator != 0 && self.numerator != 0 && self.numerator < self.denominator {
            return Ok(());
        }

        Err(BaseError::InvalidFee)
    }
}