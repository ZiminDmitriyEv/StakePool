use near_sdk::AccountId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountRegistry {
    pub owner_id: AccountId,
    pub manager_id: AccountId,
    pub self_fee_receiver_account_id: AccountId,
    pub partner_fee_receiver_account_id: AccountId
}