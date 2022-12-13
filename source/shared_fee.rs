use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::clone::Clone;
use super::fee::Fee;

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SharedFee {                                          // СДелать изменения Фии через некоторое количество эпох
    pub self_fee: Fee,
    pub partner_fee: Option<Fee>
}