use cosmwasm_std::{Response, StdError};
use cw20_base::ContractError as Cw20ContractError;
use cw_utils::ParseReplyError;
use thiserror::Error;

/// Describes router-test contract errors!
#[derive(Error, Debug, PartialEq)]
pub enum CwTokenError {
    /// Standard library
    #[error("{0}")]
    Std(#[from] StdError),

    /// Invalid Reply ID Error
    #[error("invalid reply id")]
    InvalidReplyId {},

    /// CW Utils Parsing library
    #[error("{0}")]
    ParseReplyError(#[from] ParseReplyError),

    /// CW20 Contract
    #[error("{0}")]
    Cw20ContractError(#[from] Cw20ContractError),
}

impl From<CwTokenError> for StdError {
    fn from(e: CwTokenError) -> Self {
        Self::generic_err(e.to_string())
    }
}

/// CW token Result type
pub type CwTokenResult<T> = Result<T, CwTokenError>;

/// CW Token Response type
pub type CwTokenResponse = CwTokenResult<Response>;
