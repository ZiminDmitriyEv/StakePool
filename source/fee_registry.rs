use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use super::shared_fee::SharedFee;

#[derive(BorshDeserialize, BorshSerialize)]      // TODO попробовать убрать клон
pub struct FeeRegistry {
    pub reward_fee: Option<SharedFee>,
    pub instant_withdraw_fee: Option<SharedFee>
}