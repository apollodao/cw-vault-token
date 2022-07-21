use cosmwasm_std::{Reply, StdError, StdResult, SubMsgResponse};

/// Unwrap a `Reply` object to extract the response
/// Move to apollo protocol utils.rs
pub(crate) fn unwrap_reply(reply: &Reply) -> StdResult<SubMsgResponse> {
    reply
        .clone()
        .result
        .into_result()
        .map_err(StdError::generic_err)
}
