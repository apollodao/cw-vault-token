use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Uint128};
use cw_storage_plus::Item;
use std::fmt::Display;

use crate::{CwTokenResponse, CwTokenResult};

/// A trait encapsulating the behavior necessary for instantiation of a token.
pub trait Instantiate: Sized {
    /// Instantiate a new token. This function should be called in the `instantiate`
    /// entry point of the contract before calling Self::save_token` in the `reply`
    /// entry point, to instantiate a new token and save it to storage.
    ///
    /// ## Arguments
    /// - `init_info`: The information needed to instantiate the token as a Binary.
    ///        It is up to the implementation to deserialize this and to the caller
    ///        to serialize a proper struct matching the needs of specific implementation.
    ///        The reason this is binary is so that we don't need yet another generic
    ///        argument.
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
    ///     MyToken::instantiate(deps, to_binary(&msg.init_info)?)
    /// }
    ///
    /// #[cfg_attr(not(feature = "library"), entry_point)]
    /// pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    ///     MyToken::save_token(deps, env, reply)
    /// }
    /// ```
    fn instantiate(&self, deps: DepsMut, init_info: Binary) -> CwTokenResponse;

    /// Saves the token to the storage in the provided `item`. This function should
    /// be called in the `reply` entry point of the contract after `Self::instantiate`
    /// has been called in the `instantiate` entry point.
    ///
    /// Arguments:
    /// - reply: The reply received to the `reply` entry point.
    /// - item: The `Item` to which the token should be saved.
    ///
    /// Returns a Response containing the messages to save the instantiated token.
    fn save_token(deps: DepsMut, env: &Env, reply: &Reply, item: &Item<Self>) -> CwTokenResponse;

    //fn set_admin_addr(&mut self, addr: &Addr);
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
