use near_sdk::Balance;

pub(crate) mod cross_contract_call;
pub(crate) mod data_transfer_object;
pub(crate) mod delayed_withdrawal_info;
pub(crate) mod delayed_withdrawn_fund;
pub(crate) mod fee_registry;
pub(crate) mod fee;
pub(crate) mod fungible_token_metadata;
pub(crate) mod fungible_token;
pub(crate) mod investment_withdrawal_info;
pub(crate) mod investor_investment_info;
pub(crate) mod management_fund;
pub mod stake_pool;
pub(crate) mod stake_decreasing_kind;
pub(crate) mod storage_key;
pub(crate) mod validating_node;
pub(crate) mod validator_staking_contract_version;
pub(crate) mod validator;

pub const EPOCH_QUANTITY_TO_DELAYED_WITHDRAWAL: u64 = 8;
pub const MAXIMUM_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME: u8 = 64;
pub const MAXIMUM_NUMBER_OF_TGAS: u64 = 300;
pub const MINIMUM_ATTACHED_DEPOSIT: Balance = 1;


// const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(35 * TGAS + GAS_FOR_RESOLVE_TRANSFER.0);  TODO ВОт так сделатьs