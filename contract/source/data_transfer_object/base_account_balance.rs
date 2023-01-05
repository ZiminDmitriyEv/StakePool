use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BaseAccountBalance {
    pub token_balance: U128,
    pub common_near_balance: U128,
    pub classic_near_balance: U128
}