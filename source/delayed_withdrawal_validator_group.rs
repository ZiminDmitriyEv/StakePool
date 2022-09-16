use near_sdk::borsh::{self, BorshSerialize, BorshDeserialize};
use near_sdk::serde::{Deserialize, Serialize};

/// Do not change the order of variants.
/// The number of options must be less than or equal to 256 (1 byte).
///
/// The number of groups is equal to the number of epochs required by a classic
/// (https://github.com/near/core-contracts/tree/master/staking-pool) staking contract
/// to make a delayed unsatke.
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum DelayedWithdrawalValidatorGroup {
    First,
    Second,
    Third,
    Fourth
}

impl DelayedWithdrawalValidatorGroup {
    pub fn set_next(&mut self) {
        match *self {
            Self::First => {
                *self = Self::Second;
            }
            Self::Second => {
                *self = Self::Third;
            }
            Self::Third => {
                *self = Self::Fourth;
            }
            Self::Fourth => {
                *self = Self::First;
            }
        }
    }
}