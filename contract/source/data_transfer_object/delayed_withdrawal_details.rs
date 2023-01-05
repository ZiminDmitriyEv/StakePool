use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DelayedWithdrawalDetails {
    pub epoch_quantity_to_take_delayed_withdrawal: u64,
    pub near_amount: U128
}