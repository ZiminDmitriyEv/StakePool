use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Fund {
    /// Near amount required for distribution on validators by pool.
    pub classic_unstaked_balance: U128,
    /// Near amount already distributed on validators by pool.
    pub classic_staked_balance: U128,
    /// Near amount already distributed on validators by investors.
    pub investment_staked_balance: U128,
    /// Near amount already distributed on validators by pool and investors.
    pub common_staked_balance: U128,
    /// Common management near amount.
    pub common_balance: U128
}