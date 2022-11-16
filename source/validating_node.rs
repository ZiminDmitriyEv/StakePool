use near_sdk::{env, StorageUsage, AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, LookupMap};
use super::investor_investment_info::InvestorInvestmentInfo;
use super::MAXIMUM_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME;
use super::storage_key::StorageKey;
use super::validator_info::ValidatorInfo;
use super::validator_staking_contract_version::ValidatorStakingContractVersion;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatingNode {
    pub validator_registry: UnorderedMap<AccountId, ValidatorInfo>,
    /// Registry of Investors who are allowed to make an deposit/withdrawal directly on/from the validator.
    pub investor_investment_registry: LookupMap<AccountId, InvestorInvestmentInfo>,
    pub validators_quantity: u64,                                       // TODO TODO TODO TODO TODO УБРАТЬ, ТАК КАК МОЖНО ВЗЯТЬ ИЗ АНОРДРЕД МЭп
    pub validators_maximum_quantity: Option<u64>,
    pub preffered_validtor: Option<AccountId>,
    pub quantity_of_validators_updated_in_current_epoch: u64,
    /// In bytes.
    pub storage_usage_per_validator: StorageUsage,
    /// In bytes.
    pub storage_usage_per_investor: StorageUsage,
    /// In bytes.
    pub storage_usage_per_distribution: StorageUsage
}

impl ValidatingNode {
    pub fn new(validators_maximum_quantity: Option<u64>) -> Self {
        Self {
            validator_registry: Self::initialize_validator_registry(),
            investor_investment_registry: Self::initialize_investor_investment_registry(),
            validators_quantity: 0,
            validators_maximum_quantity,
            preffered_validtor: None,
            quantity_of_validators_updated_in_current_epoch: 0,
            storage_usage_per_validator: Self::calculate_storage_usage_per_additional_validator(),
            storage_usage_per_investor: Self::calculate_storage_usage_per_additional_investor_investment(),
            storage_usage_per_distribution: Self::calculate_storage_usage_per_additional_distribution()
        }
    }

    fn calculate_storage_usage_per_additional_validator() -> StorageUsage {      // TODO СТоит ли сделать одинаковые методы через дженерик или макрос?
        let mut validator_registry = Self::initialize_validator_registry();

        let initial_storage_usage = env::storage_usage();

        let account_id = AccountId::new_unchecked("a".repeat(MAXIMUM_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        validator_registry.insert(
            &account_id, &ValidatorInfo::new(ValidatorStakingContractVersion::Classic, false)
        );

        env::storage_usage() - initial_storage_usage
    }

    fn calculate_storage_usage_per_additional_investor_investment() -> StorageUsage {
        let mut investor_registry = Self::initialize_investor_investment_registry();

        let initial_storage_usage = env::storage_usage();

        let account_id = AccountId::new_unchecked("a".repeat(MAXIMUM_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        investor_registry.insert(&account_id, &InvestorInvestmentInfo::new(account_id.clone()));

        env::storage_usage() - initial_storage_usage
    }

    fn calculate_storage_usage_per_additional_distribution() -> StorageUsage {
        let account_id = AccountId::new_unchecked("a".repeat(MAXIMUM_NUMBER_OF_CHARACTERS_IN_ACCOUNT_NAME as usize));

        let mut distribution_registry = InvestorInvestmentInfo::initialize_distribution_registry(account_id.clone());

        let initial_storage_usage = env::storage_usage();

        distribution_registry.insert(&account_id, &0);

        env::storage_usage() - initial_storage_usage
    }

    fn initialize_validator_registry() -> UnorderedMap<AccountId, ValidatorInfo> {
        UnorderedMap::new(StorageKey::ValidatorRegistry)
    }

    fn initialize_investor_investment_registry() -> LookupMap<AccountId, InvestorInvestmentInfo> {
        LookupMap::new(StorageKey::InvestorRegistry)
    }
}