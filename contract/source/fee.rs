use near_sdk::Balance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env;
use near_sdk::serde::{Deserialize, Serialize};
use std::clone::Clone;
use uint::construct_uint;

construct_uint! {
    pub struct U256(4);
}

#[derive(Clone, Debug, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Fee {
    pub numerator: u64,
    pub denominator: u64
}

impl Fee {
    pub fn assert_valid(&self) {
        if self.denominator == 0 || self.numerator == 0 || self.numerator >= self.denominator {
            env::panic_str("Fee is not valid.");
        }
    }

    pub fn multiply(&self, value: Balance) -> Balance {
        (
            U256::from(self.numerator) * U256::from(value)
            / U256::from(self.denominator)
        ).as_u128()
    }
}