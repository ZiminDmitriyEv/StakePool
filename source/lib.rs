use near_sdk::Balance;

pub mod stake_pool;
pub(crate) mod account_registry;
pub(crate) mod cross_contract_call;
pub(crate) mod data_transfer_object;
pub(crate) mod delayed_withdrawal;
pub(crate) mod delayed_withdrawn_fund;
pub(crate) mod fee_registry;
pub(crate) mod fee;
pub(crate) mod fund;
pub(crate) mod fungible_token_metadata;
pub(crate) mod fungible_token;
pub(crate) mod investment_withdrawal;
pub(crate) mod investor_investment;
pub(crate) mod stake_decreasing_kind;
pub(crate) mod staking_contract_version;
pub(crate) mod storage_key;
pub(crate) mod validating;
pub(crate) mod validator;

pub const EPOCH_QUANTITY_TO_DELAYED_WITHDRAWAL: u64 = 8;
pub const MAXIMUM_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME: u8 = 64;
pub const MAXIMUM_NUMBER_OF_TGAS: u64 = 300;
pub const MINIMUM_ATTACHED_DEPOSIT: Balance = 1;


// const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(35 * TGAS + GAS_FOR_RESOLVE_TRANSFER.0);  TODO ВОт так сделатьs