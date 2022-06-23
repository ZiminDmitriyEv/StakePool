use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use super::fee::Fee;

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AggregatedInformation {
    /// YoctoNear amount required for distribution on validators.
    available_for_staking_balance: U128,

    /// YoctoNear amount already distributed on validators.
    staked_balance: U128,

    /// Minted amount of token.
    token_total_supply: U128,

    /// Stakers quantity.
    token_accounts_quantity: u64,

    /// YoctoNear amount of rewards from validators.
    total_rewards_from_validators_yocto_near_amount: U128,

    /// Fee charged by the pool when receiving rewards from validators.
    rewards_fee: Option<Fee>
}

impl AggregatedInformation {
    pub fn new(
    available_for_staking_balance: U128,
    staked_balance: U128,
    token_total_supply: U128,
    token_accounts_quantity: u64,
    total_rewards_from_validators_yocto_near_amount: U128,
    rewards_fee: Option<Fee>
    ) -> Self {
        Self {
            available_for_staking_balance,
            staked_balance,
            token_total_supply,
            token_accounts_quantity,
            total_rewards_from_validators_yocto_near_amount,
            rewards_fee
        }
    }
}