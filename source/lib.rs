pub(crate) mod aggregated_information_dto;
pub(crate) mod delayed_withdrawal_info;
pub(crate) mod delayed_withdrawn_fund;
pub(crate) mod fee_registry;
pub(crate) mod fee;
pub(crate) mod fungible_token_metadata;
pub(crate) mod fungible_token;
pub(crate) mod investment_withdrawal_info;
pub(crate) mod investor_info;
pub(crate) mod management_fund;
pub(crate) mod requested_to_withdrawal_fund;
pub mod stake_pool;
pub(crate) mod stake_decreasing_kind;
pub(crate) mod storage_key;
pub(crate) mod validating_node;
pub(crate) mod validator_info_dto;
pub(crate) mod validator_info;
pub(crate) mod validator_staking_contract_version;
pub(crate) mod xcc_staking_pool;


pub const EPOCH_QUANTITY_TO_DELAYED_WITHDRAWAL: u64 = 8;
pub const MAXIMUM_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME: u8 = 64;
pub const MAXIMUM_NUMBER_OF_TGAS: u64 = 300;