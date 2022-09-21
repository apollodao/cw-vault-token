use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Uint128};

use std::fmt::Display;

use crate::{CwTokenResponse, CwTokenResult};

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

pub trait Token: Display {
    fn transfer<A: Into<String>>(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: A,
        amount: Uint128,
    ) -> CwTokenResponse;

    fn query_balance<A: Into<String>>(&self, deps: Deps, address: A) -> CwTokenResult<Uint128>;

    fn is_native() -> bool;
}

pub trait Send {
    fn send<A: Into<String>>(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract: A,
        amount: Uint128,
        msg: Binary,
    ) -> CwTokenResponse;

    fn send_from<A: Into<String>>(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        owner: A,
        contract: A,
        amount: Uint128,
        msg: Binary,
    ) -> CwTokenResponse;
}

pub trait TransferFrom {
    fn transfer_from<A: Into<String>, B: Into<String>>(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        _from: A,
        _to: B,
        _amount: Uint128,
    ) -> CwTokenResponse;
}

pub trait Mint {
    fn mint<A: Into<String>, B: Into<String>>(
        &self,
        deps: DepsMut,
        sender: A,
        recipient: B,
        amount: Uint128,
    ) -> CwTokenResponse;
}

pub trait Burn {
    fn burn<A: Into<String>>(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        sender: A,
        amount: Uint128,
    ) -> CwTokenResponse;
}
