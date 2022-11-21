use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

/// Do not change the order of variants.
/// The number of options must be less than or equal to 256 (1 byte).
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub enum StakingContractVersion {
    // https://github.com/near/core-contracts/tree/master/staking-pool
    Classic
}