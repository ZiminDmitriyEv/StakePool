use near_contract_standards::fungible_token::metadata::FT_METADATA_SPEC;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(BorshDeserialize, BorshSerialize, Clone, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenMetadata {
    spec: String,
    name: String,
    symbol: String,
    icon: Option<String>,
    reference: Option<String>,
    reference_hash: Option<Base64VecU8>,
    decimals: u8,
}

impl FungibleTokenMetadata {
    pub fn new(
        name: String,
        symbol: String,
        icon: Option<String>,
        reference: Option<String>,
        reference_hash: Option<Base64VecU8>,
        decimals: u8,
    ) -> Self {
        Self {
            spec: FT_METADATA_SPEC.to_string(),
            name,
            symbol,
            icon,
            reference,
            reference_hash,
            decimals
        }
    }

    pub fn is_valid(&self) -> bool {
        if !(self.reference.is_some() == self.reference_hash.is_some()) {
            return false;
        }
        if let Some(reference_hash) = &self.reference_hash {
            if reference_hash.0.len() != 32 {
                return false;
            }
        }

        true
    }
}
