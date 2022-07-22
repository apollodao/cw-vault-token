use std::fmt::Display;

use cosmwasm_std::{
    Addr, Api, Binary, QuerierWrapper, Reply, Response, StdResult, Storage, SubMsg, Uint128,
};
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

    fn set_admin_addr(&mut self, addr: &Addr);
}

pub trait Token: Display {
    fn transfer<A: Into<String>>(&self, to: A, amount: Uint128) -> StdResult<Response>;

    fn query_balance<A: Into<String>>(
        &self,
        querier: &QuerierWrapper,
        address: A,
    ) -> StdResult<Uint128>;

    fn is_native() -> bool;
}

pub trait Send {
    fn send<A: Into<String>>(&self, to: A, amount: Uint128, msg: Binary) -> StdResult<Response>;
}

pub trait TransferFrom {
    fn transfer_from<A: Into<String>, B: Into<String>>(
        &self,
        _from: A,
        _to: B,
        _amount: Uint128,
    ) -> StdResult<Response> {
        unimplemented!()
    }
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

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------
