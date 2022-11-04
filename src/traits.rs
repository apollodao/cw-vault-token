use cosmwasm_std::{ Addr, Binary, Deps, DepsMut, Env, MessageInfo, StdResult, Uint128 };

use std::fmt::Display;

use crate::{ CwTokenResponse, CwTokenResult };

/// Combined trait for implementations that can be used as a vault token.
pub trait VaultToken: Instantiate + Mint + Burn + Receive + Display {
    /// ## Description
    /// Query the balance of the vault token for `address`.
    /// # Errors
    ///
    /// Will return `Err` if `address` does not exist on the contract data.
    fn query_balance<A: Into<String>>(&self, deps: Deps, address: A) -> CwTokenResult<Uint128>;

    /// ## Description
    /// Query the total supply of the vault token.
    /// # Errors
    ///
    /// Will return `Err` if broken data.
    fn query_total_supply(&self, deps: Deps) -> CwTokenResult<Uint128>;
}

/// A trait encapsulating the behavior necessary for instantiation of a token.
pub trait Instantiate {
    /// ## Description
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
    /// ## Example (pseudocode)
    /// ```ignore
    /// #[cfg_attr(not(feature = "library"), entry_point)]
    /// pub fn instantiate(
    ///     deps: DepsMut,
    ///     env: Env,
    ///     info: MessageInfo,
    ///     msg: InstantiateMsg,
    /// ) -> Result<Response, ContractError> {
    ///     let my_token = MyToken::new(..);
    ///     my_token.instantiate(deps, to_binary(&msg.init_info)?)
    /// }
    /// ```
    /// # Errors
    ///
    /// Will return `Err` if `msg.init_info` has incorrect data.
    fn instantiate(&self, deps: DepsMut, init_info: Option<Binary>) -> CwTokenResponse;
}

/// Defines the Minting rules of a token
pub trait Mint {
    /// ## Description
    /// Mints `amount` new vault tokens to the `recipient` address.
    /// The contract should validate that the recipient is allowed to do this before
    /// calling the function, i.e. make sure that the recipient has sent sufficient
    /// assets to the vault, or perform a `transfer_from`, or similar.
    /// # Errors
    ///
    /// Will return `Err` if `recipient` does not have priviledges.
    fn mint(&self, deps: DepsMut, env: &Env, recipient: &Addr, amount: Uint128) -> CwTokenResponse;
}

/// Defines the Burning rules of a token
pub trait Burn {
    /// ## Description
    /// Burns vault tokens from the contract's balance.
    /// # Errors
    ///
    /// Will return `Err` if `amount` is greater than contract balance.
    fn burn(&self, deps: DepsMut, env: &Env, amount: Uint128) -> CwTokenResponse;
}

/// Defines the Receive behaviour of a token
pub trait Receive {
    /// ## Description
    /// Receive the vault token into the contracts balance, or validate that they
    /// have already been received.
    /// E.g. if it is a native token, assert that this amount exists in info.funds,
    /// and if it is a CW4626, transfer from the caller's balance into the contract's.
    /// We do this so that we can call this at the beginning of a contract `ExecuteMsg`
    /// handler, and then know that after this the behavior is the same for both for
    /// both implementations.
    /// # Errors
    ///
    /// Will return `Err` if `MessageInfo` has worng data.
    fn receive_vault_token(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        amount: Uint128
    ) -> StdResult<()>;
}