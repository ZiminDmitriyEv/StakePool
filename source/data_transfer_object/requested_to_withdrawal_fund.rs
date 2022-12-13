use near_sdk::AccountId;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RequestedToWithdrawalFund {
    pub classic_near_amount: U128,
    pub investment_near_amount: U128,
    pub investment_withdrawal_registry: Vec<(AccountId, U128)>
}