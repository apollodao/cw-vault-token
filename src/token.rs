use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Env, MessageInfo, StdResult, Uint128};

use std::fmt::Display;

use crate::{cw4626::Cw4626, osmosis::OsmosisDenom, CwTokenResponse, CwTokenResult};

/// Combined trait for implementations that can be used as a vault token.
///
/// Instantiate is not required here since Cw4626 does not require any specific
/// instantiation. Osmosis does require instantiation, but this can simply be
/// handled in the top contract.rs where we know that the VaultToken is an
/// OsmosisDenom.
pub trait VaultToken: Instantiate + Mint + Burn + Token + Display + AssertReceived {}

/// We currently only implement VaultToken for OsmosisDenom and Cw4626, because
/// we use AssertReceived which cannot be implemented for CW20 in a clean way
/// since there we need to do TransferFrom to transfer the CW20 tokens to the
/// vault contract.
impl VaultToken for OsmosisDenom {}
impl VaultToken for Cw4626 {}

/// A trait encapsulating the behavior necessary for instantiation of a token.
pub trait Instantiate {
    /// Instantiate a new token. This function should be called in the `instantiate`
    /// entry point of the contract, to instantiate a new token.
    ///
    /// ## Arguments
    /// - `init_info`: The information needed to instantiate the token as a Binary.
    ///        It is up to the implementation to deserialize this and to the caller
    ///        to serialize a proper struct matching the needs of specific implementation.
    ///        The reason this is binary is so that we don't need yet another generic
    ///        argument. It is optional as not all implementations need info to be
    ///        able to initialize.
    ///
    /// ## Returns
    /// Returns a Response containing the messages to instantiate the token.
    ///
    /// ## Example
    /// ```
    /// #[cfg_attr(not(feature = "library"), entry_point)]
    /// pub fn instantiate(
    ///     deps: DepsMut,
    ///     env: Env,
    ///     info: MessageInfo,
    ///     msg: InstantiateMsg,
    /// ) -> Result<Response, ContractError> {
    ///     let my_token = MyToken::new(...);
    ///     my_token.instantiate(deps, to_binary(&msg.init_info)?)
    /// }
    /// ```
    fn instantiate(&self, deps: DepsMut, init_info: Option<Binary>) -> CwTokenResponse;
}

pub trait Token {
    fn transfer<A: Into<String>>(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: A,
        amount: Uint128,
    ) -> CwTokenResponse;

    fn query_balance<A: Into<String>>(&self, deps: Deps, address: A) -> CwTokenResult<Uint128>;

    fn query_total_supply(&self, deps: Deps) -> CwTokenResult<Uint128>;

    fn is_native() -> bool;
}

pub trait Mint {
    fn mint(&self, deps: DepsMut, env: &Env, recipient: &Addr, amount: Uint128) -> CwTokenResponse;
}

pub trait Burn {
    fn burn(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        owner: &Addr,
        amount: Uint128,
    ) -> CwTokenResponse;
}

// Validates that the `amount` amount of tokens were received by the contract.
// E.g. if it is a native token, assert that this amount exists in info.funds,
// and that if it is a Cw4626 that the user has this amount of tokens in their
// balance.
pub trait AssertReceived {
    fn assert_received(&self, deps: Deps, info: &MessageInfo, amount: Uint128) -> StdResult<()>;
}
