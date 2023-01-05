use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageStakingPrice {
    pub per_delayed_withdrawal_fund_delayed_withdrawal: U128,
    pub per_delayed_withdrawal_fund_investment_withdrawal: U128,
    pub per_fungible_token_account: U128,
    pub per_validating_node_validator: U128,
    pub per_validating_node_investor: U128,
    pub per_validating_node_distribution: U128
}