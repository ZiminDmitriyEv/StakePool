use near_sdk::EpochHeight;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EpochHeightRegistry {
    pub pool_epoch_height: EpochHeight,
    pub network_epoch_height: EpochHeight
}