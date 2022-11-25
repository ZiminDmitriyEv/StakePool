use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use std::clone::Clone;
use super::fee::Fee;

#[derive(Clone, BorshDeserialize, BorshSerialize)]
pub struct SharedFee {
    pub self_fee: Fee,
    pub partner_fee: Option<Fee>
}