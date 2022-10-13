use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::U128;

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenRegistryDto {
    pub classic_token_balance: U128,
    pub investment_token_balance: U128
}