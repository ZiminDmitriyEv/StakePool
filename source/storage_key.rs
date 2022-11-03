use near_sdk::AccountId;
use near_sdk::borsh::{self, BorshSerialize};
use near_sdk::BorshStorageKey;

/// Do not change the order of variants.
/// The number of options must be less than or equal to 256 (1 byte).
#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {                               // TODO Придумать, как их назвать. Как назвать LazyOption<UnorderedMap>  - все должны быть разные.
    InvestorRegistry,
    FungibleToken,
    FungibleTokenMetadata,                 // TODO проверить, что они используются по одному разу
    ValidatorRegistry,
    DelayedWithdrawnFund,
    DistributionRegistry {
        investor_account_id: AccountId
    }
}