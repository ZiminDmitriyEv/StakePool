use crate::fee::Fee;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FeeRegistryLight {
    pub reward_fee: Option<Fee>,
    pub instant_withdraw_fee: Option<Fee>
}