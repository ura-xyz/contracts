use cosmwasm_std::{DecimalRangeExceeded, OverflowError, StdError};
use thiserror::Error;

use cw_controllers::AdminError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    GenericError(String),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Cannot end epoch yet, need to wait until {0}")]
    CannotEndEpoch(u64),

    #[error("User does not have any tokens to vest")]
    NotEnoughTokens(),

    #[error("{0}")]
    OverflowError(OverflowError),

    #[error("{0}")]
    DecimalRangeExceeded(DecimalRangeExceeded),
}
