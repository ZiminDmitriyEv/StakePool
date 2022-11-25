use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use super::base_account_balance::BaseAccountBalance;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountBalance {
    pub base_account_balance: Option<BaseAccountBalance>,
    pub investment_account_balance: Option<U128>
}