use near_contract_standards::fungible_token::metadata::FT_METADATA_SPEC;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(BorshDeserialize, BorshSerialize, Clone, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<Base64VecU8>,
    pub decimals: u8,
}

impl FungibleTokenMetadata {
    const TOKEN_IMAGE_PATH: &'static str = "todo_path";
    const TOKEN_NAME:  &'static str = "todo_name";
    const TOKEN_SYMPOL:  &'static str = "todo_symbol";
    const TOKEN_DECIMALS: u8 = 24;

    pub fn new() -> Self {
        Self {
            spec: FT_METADATA_SPEC.to_string(),
            name: Self::TOKEN_NAME.to_string(),
            symbol: Self::TOKEN_SYMPOL.to_string(),
            icon: Some(Self::TOKEN_IMAGE_PATH.to_string()),
            reference: None,                                            // TODO TODO TODO TODO TODO что это
            reference_hash: None,                                       // TODO TODO TODO TODO TODO что это
            decimals: Self::TOKEN_DECIMALS
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
