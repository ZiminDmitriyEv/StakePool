use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::clone::Clone;
use super::base_error::BaseError;
use uint::construct_uint;

construct_uint! {
    pub struct U256(4);
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Fee {
    pub numerator: u64,
    pub denominator: u64
}

impl Fee {
    pub fn assert_valid(&self) -> Result<(), BaseError> {
        if self.denominator != 0 && self.numerator != 0 && self.numerator < self.denominator {
            return Ok(());
        }

        Err(BaseError::InvalidFee)
    }

    // TODO нужно ли сделать безопасно? Что здесь по огругению
    pub fn multiply(&self, value: Balance) -> Balance {
        (
            U256::from(self.numerator) * U256::from(value)
            / U256::from(self.denominator)
        ).as_u128()
    }
}