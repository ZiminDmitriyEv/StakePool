use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use super::account_balance::AccountBalance;
use super::delayed_withdrawal_details::DelayedWithdrawalDetails;
use super::fee_registry_light::FeeRegistryLight;
use super::fund::Fund;
use super::storage_staking_price::StorageStakingPrice;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Full {
    pub storage_staking_price: StorageStakingPrice,
    pub fund: Fund,
    pub account_balance: AccountBalance,
    pub delayed_withdrawal_details: Option<DelayedWithdrawalDetails>,
    pub total_token_supply: U128,
    pub fee_registry_light: FeeRegistryLight
}