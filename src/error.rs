use cosmwasm_std::{Response, StdError};
use cw20_base::ContractError;
use thiserror::Error;

/// ## Description
/// This enum describes router-test contract errors!
#[derive(Error, Debug, PartialEq)]

pub enum CwTokenError {
    #[error("{0}")]
    Std(#[from] StdError),

    /// Invalid Reply ID Error
    #[error("invalid reply id")]
    InvalidReplyId {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.31/thiserror/ for details.
}

impl From<ContractError> for CwTokenError {
    fn from(x: ContractError) -> Self {
        CwTokenError::Std(StdError::generic_err(x.to_string()))
    }
}

pub type CwTokenResult<T> = Result<T, CwTokenError>;
pub type CwTokenResponse = CwTokenResult<Response>;
