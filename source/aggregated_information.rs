use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use super::fee::Fee;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AggregatedInformation {                                         // TODO ВСе ДТО ВЫНЕСТИ В модуль.
    /// YoctoNear amount required for distribution on validators.
    pub available_for_staking_balance: U128,

    /// YoctoNear amount already distributed on validators.
    pub staked_balance: U128,

    /// Minted amount of token.
    pub token_total_supply: U128,

    /// Stakers quantity.
    pub token_accounts_quantity: u64,

    /// YoctoNear amount of rewards from validators.
    pub total_rewards_from_validators_yocto_near_amount: U128,

    /// Fee charged by the pool when receiving rewards from validators.
    pub rewards_fee: Option<Fee>
}