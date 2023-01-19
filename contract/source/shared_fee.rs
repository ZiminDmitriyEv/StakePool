use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::clone::Clone;
use super::fee::Fee;

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SharedFee {
    /// Fee taken from object.
    pub self_fee: Fee,
    /// Fee taken from 'self_fee".
    pub partner_fee: Option<Fee>
}