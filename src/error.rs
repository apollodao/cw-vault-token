use cosmwasm_std::{Response, StdError};
use cw20_base::ContractError as Cw20ContractError;
use cw_utils::ParseReplyError;
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

    #[error("{0}")]
    ParseReplyError(#[from] ParseReplyError),

    #[error("{0}")]
    Cw20ContractError(#[from] Cw20ContractError),
}

pub type CwTokenResult<T> = Result<T, CwTokenError>;
pub type CwTokenResponse = CwTokenResult<Response>;
