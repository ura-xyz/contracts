use cosmwasm_std::{ConversionOverflowError, OverflowError, StdError};
use thiserror::Error;

use ura::contracts::pair::MINIMUM_LIQUIDITY_AMOUNT;

use crate::math::{MAX_AMP, MAX_AMP_CHANGE, MIN_AMP_CHANGING_TIME};

/// This enum describes stableswap pair contract errors
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Operation non supported")]
    NonSupported {},

    #[error("Invalid state")]
    InvalidState {},

    #[error("CW20 tokens can be swapped via Cw20::Send message only")]
    Cw20DirectSwap {},

    #[error("Doubling assets in asset infos")]
    DoublingAssets {},

    #[error("Event of zero transfer")]
    InvalidZeroAmount {},

    #[error("Wrong token funds sent to withdraw liquidity")]
    InvalidLiquidityToken {},

    #[error("Insufficient amount of liquidity")]
    LiquidityAmountTooSmall {},

    #[error("Provided spread amount exceeds allowed limit")]
    AllowedSpreadAssertion {},

    #[error("Operation exceeds max spread limit")]
    MaxSpreadAssertion {},

    #[error("Native token balance mismatch between the argument and the transferred")]
    AssetMismatch {},

    #[error(
        "Amp coefficient must be greater than 0 and less than or equal to {}",
        MAX_AMP
    )]
    IncorrectAmp {},

    #[error(
        "The difference between the old and new amp value must not exceed {} times",
        MAX_AMP_CHANGE
    )]
    MaxAmpChangeAssertion {},

    #[error(
        "Amp coefficient cannot be changed more often than once per {} seconds",
        MIN_AMP_CHANGING_TIME
    )]
    MinAmpChangingTimeAssertion {},

    #[error("You need to provide init params")]
    InitParamsNotFound {},

    #[error("It is not possible to provide liquidity with one token for an empty pool")]
    InvalidProvideLPsWithSingleToken {},

    #[error("The asset {0} does not belong to the pair")]
    InvalidAsset(String),

    #[error("Ask or offer asset is missed")]
    VariableAssetMissed {},

    #[error("Source and target assets are the same")]
    SameAssets {},

    #[error("Invalid number of assets. This pair support only {0} assets")]
    InvalidNumberOfAssets(usize),

    #[error("Initial liquidity must be more than {}", MINIMUM_LIQUIDITY_AMOUNT)]
    MinimumLiquidityAmountError {},

    #[error("Failed to parse or process reply message")]
    FailedToParseReply {},
}

impl From<OverflowError> for ContractError {
    fn from(o: OverflowError) -> Self {
        StdError::from(o).into()
    }
}

impl From<ConversionOverflowError> for ContractError {
    fn from(o: ConversionOverflowError) -> Self {
        StdError::from(o).into()
    }
}
