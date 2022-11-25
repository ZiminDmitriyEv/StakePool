use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Fund {
    /// Near amount already distributed on validators.
    pub staked_balance: U128,
    /// Near amount required for distribution on validators.
    pub unstaked_balance: U128,
    /// Common management near amount.
    pub common_balance: U128
}