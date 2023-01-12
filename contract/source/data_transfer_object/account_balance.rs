use near_sdk::serde::{Deserialize, Serialize};
use super::base_account_balance::BaseAccountBalance;
use super::investment_account_balance::InvestmentAccountBalance;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountBalance {
    pub base_account_balance: Option<BaseAccountBalance>,
    pub investment_account_balance: Option<InvestmentAccountBalance>
}