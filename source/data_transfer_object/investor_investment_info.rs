use near_sdk::AccountId;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct InvestorInvestmentInfo {
    pub distribution_registry: Vec<(AccountId, U128)>,
    pub staked_balance: U128
}