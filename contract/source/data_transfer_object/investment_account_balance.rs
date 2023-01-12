use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct InvestmentAccountBalance {
    pub near_balance: U128,
    pub near_balance_token_coverage: U128
}