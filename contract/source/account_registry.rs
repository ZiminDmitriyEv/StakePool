use near_sdk::AccountId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountRegistry {
    pub owner_id: AccountId,
    pub manager_id: AccountId,
    /// Id of account, that will receive 'self_fee'.
    pub self_fee_receiver_account_id: AccountId,
    /// Id of account, that will receive 'partner_fee'.
    pub partner_fee_receiver_account_id: AccountId
}