use near_sdk::borsh::{self, BorshSerialize};
use near_sdk::BorshStorageKey;


/// Do not change the order of variants.
/// The number of options must be less than or equal to 256 (1 byte).
#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    FungibleToken1,
    FungibleTokenMetadata1,
    ValidatorNode1,
    ValidatorNode2,
}