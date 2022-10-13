use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::Balance;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenRegistry {
    pub classic_token_balance: Balance,
    pub investment_token_balance: Balance
}