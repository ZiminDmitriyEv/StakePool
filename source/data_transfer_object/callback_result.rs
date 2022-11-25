use near_sdk::EpochHeight;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CallbackResult {
    pub is_success: bool,
    pub network_epoch_height: EpochHeight
}