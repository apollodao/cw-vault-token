use cosmwasm_std::{Api, Binary, Reply, Response, StdResult, Storage, SubMsg, Uint128};
use cw_storage_plus::Item;
use serde::{de::DeserializeOwned, Serialize};

use crate::CwTokenError;
pub trait Instantiate<A: Serialize + DeserializeOwned>: Sized {
    fn instantiate_msg(&self) -> StdResult<SubMsg>;

    fn save_asset(
        storage: &mut dyn Storage,
        api: &dyn Api,
        reply: &Reply,
        item: Item<A>,
    ) -> Result<Response, CwTokenError>;
}

pub trait Send {
    fn send<A: Into<String>>(&self, to: A, amount: Uint128, msg: Binary) -> StdResult<Response>;
}

pub trait Transfer {
    fn transfer<A: Into<String>>(&self, to: A, amount: Uint128) -> StdResult<Response>;
    fn transfer_from<A: Into<String>, B: Into<String>>(
        &self,
        from: A,
        to: B,
        amount: Uint128,
    ) -> StdResult<Response>;
}

pub trait Mint {
    fn mint<A: Into<String>, B: Into<String>>(
        &self,
        sender: A,
        recipient: B,
        amount: Uint128,
    ) -> StdResult<Response>;

    fn is_mintable() -> bool {
        true
    }
}

pub trait Burn {
    fn burn<A: Into<String>>(&self, sender: A, amount: Uint128) -> StdResult<Response>;

    fn is_burnable() -> bool {
        true
    }
}

pub trait IsNative {
    fn is_native() -> bool;
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------
