use near_sdk::AccountId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountRegistry {
    pub owner_id: AccountId,
    pub manager_id: AccountId,
    pub rewards_receiver_account_id: AccountId,
    pub everstake_rewards_receiver_account_id: AccountId,
}

impl AccountRegistry {
    pub fn new(
        owner_id: AccountId,
        manager_id: AccountId,
        rewards_receiver_account_id: AccountId,
        everstake_rewards_receiver_account_id: AccountId,
    ) -> Self {
        Self {
            owner_id,
            manager_id,
            rewards_receiver_account_id,
            everstake_rewards_receiver_account_id
        }
    }
}