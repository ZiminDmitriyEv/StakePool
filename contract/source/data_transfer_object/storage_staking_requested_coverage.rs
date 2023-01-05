use near_sdk::AccountId;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageStakingRequestedCoverage {
    pub per_method_deposit: U128,
    pub per_method_deposit_on_validator: Option<(U128, Vec<(AccountId, U128)>)>,
    pub per_method_delayed_withdraw: U128,
    pub per_method_delayed_withdraw_from_validator: Option<(U128, Vec<(AccountId, U128)>)>
}