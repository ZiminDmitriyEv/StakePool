use std::error::Error;
use std::fmt::Display;
use std::fmt::Error as FmtError;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum BaseError {        // TODO описать Контектс там, где он нцжен для понимания информации.
    Logic,
    ContractStateAlreadyInitialized,
    UnauthorizedManagementOnlyByManager,
    UnauthorizedManagement,
    CalculationOwerflow,
    ZeroIncreasing,
    ZeroDecreasing,
    InvalidFee,
    SynchronizedEpoch,
    DesynchronizedEpoch,
    InvalidFungibleTokenMetadata,
    TokenAccountAlreadyRegistered,
    TokenAccountIsNotRegistered,
    UnregisterTokenAccountWithNonZeroTokenBalance,
    InsufficientTokenAccountBalance,
    InsufficientNearDeposit,
    InsufficientNearDepositForStorageStaking,
    InsufficientTokenDeposit,
    ValidatorAccountIsAlreadyRegistered,
    ValidatorAccountIsNotRegistered,
    ValidatorAccountsMaximumQuantityExceeding,
    ValidatorAccountsZeroQuantity,
    InsufficientAvailableForStakingBalance,
    InsufficientStakedBalance,
    ValidatorInfoAlreadyUpdated,
    SomeValidatorInfoDoesNotUpdated,
    RemovingValidatorWithExistingBalance,
    SameAccountId,
    DelayedWithdrawalAccountAlreadyRegistered
}

impl Error for BaseError {}     // TODO Выводить сразу в лог с паникойй. Убрать ошибки?

impl Display for BaseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), FmtError> {
        match *self {
            Self::Logic => {
                formatter.write_str("Logic error.")?;
            }
            Self::ContractStateAlreadyInitialized => {
                formatter.write_str("The contract state has already been initialized.")?;
            }
            Self::UnauthorizedManagementOnlyByManager => {
                formatter.write_str("This action is managed by only the pool manager.")?;
            }
            Self::UnauthorizedManagement => {
                formatter.write_str("This action is managed by the pool owner or pool manager.")?;
            }
            Self::CalculationOwerflow => {
                formatter.write_str("Calculation owerflow.")?;
            }
            Self::ZeroIncreasing => {
                formatter.write_str("Increasing with zero value.")?;
            }
            Self::ZeroDecreasing => {
                formatter.write_str("Decreasing with zero value.")?;
            }
            Self::InvalidFee => {
                formatter.write_str("Invalid fee.")?;
            }
            Self::SynchronizedEpoch => {
                formatter.write_str("The epoch is synchronized.")?;
            }
            Self::DesynchronizedEpoch => {
                formatter.write_str("The epoch is desynchronized.")?;
            }
            Self::InvalidFungibleTokenMetadata => {
                formatter.write_str("Invalid fungible roken metadata.")?;
            }
            Self::TokenAccountAlreadyRegistered => {
                formatter.write_str("The token account is already registered.")?;
            }
            Self::TokenAccountIsNotRegistered => {
                formatter.write_str("The token account is not registered yet.")?;
            }
            Self::UnregisterTokenAccountWithNonZeroTokenBalance => {
                formatter.write_str("Attempt to delete a token account with a non-zero balance.")?;
            }
            Self::InsufficientTokenAccountBalance => {
                formatter.write_str("Insufficient token account balance.")?;
            }
            Self::InsufficientNearDeposit => {
                formatter.write_str("Insufficient NEAR deposit.")?;
            }
            Self::InsufficientNearDepositForStorageStaking => {
                formatter.write_str("Insufficient NEAR deposit for storage staking.")?;
            }
            Self::InsufficientTokenDeposit => {
                formatter.write_str("Insufficient token deposit.")?;
            },
            Self::ValidatorAccountIsAlreadyRegistered => {
                formatter.write_str("Unable to add validator to the pool. Validator is already under pool management.")?;
            }
            Self::ValidatorAccountIsNotRegistered => {
                formatter.write_str("The validator account is not registered.")?;
            }
            Self::ValidatorAccountsMaximumQuantityExceeding => {
                formatter.write_str("Unable to add validator to the pool. The validators maximum quantity is exceeded.")?;
            }
            Self::ValidatorAccountsZeroQuantity => {
                formatter.write_str("There are no validator accounts in the pool.")?;
            }
            Self::InsufficientAvailableForStakingBalance => {
                formatter.write_str("Insufficient available for staking balance.")?;
            }
            Self::InsufficientStakedBalance => {
                formatter.write_str("Insufficient staked balance.")?;
            }
            Self::ValidatorInfoAlreadyUpdated => {
                formatter.write_str("Validator info already updated.")?;
            }
            Self::SomeValidatorInfoDoesNotUpdated => {
                formatter.write_str("The information about some validators does not updated.")?;
            }
            Self::RemovingValidatorWithExistingBalance => {
                formatter.write_str("The validator has a non-zero balance, so it cannot be removed.")?;
            }
            Self::SameAccountId => {
                formatter.write_str("Account Ids should not be the same.")?;
            }
            Self::DelayedWithdrawalAccountAlreadyRegistered => {
                formatter.write_str("Delayed withdrawal account already registered. Please, wait a few epoch.")?;
            }
        }

        Ok(())
    }
}