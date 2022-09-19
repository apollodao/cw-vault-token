use crate::{
    token::{Burn, Mint},
    Token, TransferFrom,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg,
    WasmQuery,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_asset::AssetInfo;
use std::{convert::TryFrom, fmt::Display};

#[cw_serde]
pub struct Cw20(pub Addr);

impl Display for Cw20 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Cw20> for AssetInfo {
    fn from(cw20_asset: Cw20) -> Self {
        AssetInfo::Cw20(cw20_asset.0)
    }
}

impl TryFrom<AssetInfo> for Cw20 {
    type Error = StdError;

    fn try_from(asset_info: AssetInfo) -> StdResult<Self> {
        match asset_info {
            AssetInfo::Cw20(address) => Ok(Cw20(address)),
            AssetInfo::Native(_) => {
                Err(StdError::generic_err("Cannot convert native addr to Cw20."))
            }
            AssetInfo::Cw1155(_x, _y) => Err(StdError::generic_err(
                "Cannot convert Cw1155 asset to Cw20.",
            )),
            _ => Err(StdError::generic_err(
                "Cannot convert unknown asset to Cw20.",
            )),
        }
    }
}

// ------ Implement Instantiate for Cw20Asset ------

pub const REPLY_SAVE_CW20_ADDRESS: u64 = 14509;

impl Token for Cw20 {
    fn transfer<A: Into<String>>(&self, to: A, amount: Uint128) -> StdResult<Response> {
        Ok(
            Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: self.0.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: to.into(),
                    amount,
                })?,
                funds: vec![],
            })),
        )
    }

    fn query_balance<A: Into<String>>(
        &self,
        querier: &cosmwasm_std::QuerierWrapper,
        address: A,
    ) -> StdResult<Uint128> {
        Ok(querier
            .query::<BalanceResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: self.0.to_string(),
                msg: to_binary(&Cw20QueryMsg::Balance {
                    address: address.into(),
                })?,
            }))?
            .balance)
    }

    fn is_native() -> bool {
        false
    }
}

impl TransferFrom for Cw20 {
    fn transfer_from<A: Into<String>, B: Into<String>>(
        &self,
        from: A,
        to: B,
        amount: Uint128,
    ) -> StdResult<Response> {
        Ok(
            Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: self.0.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: from.into(),
                    recipient: to.into(),
                    amount,
                })?,
                funds: vec![],
            })),
        )
    }
}

impl Mint for Cw20 {
    fn mint<A: Into<String>, B: Into<String>>(
        &self,
        _sender: A,
        recipient: B,
        amount: Uint128,
    ) -> StdResult<Response> {
        Ok(
            Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: self.0.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: recipient.into(),
                    amount,
                })?,
                funds: vec![],
            })),
        )
    }
}

impl Burn for Cw20 {
    fn burn<A: Into<String>>(&self, _sender: A, amount: Uint128) -> StdResult<Response> {
        Ok(
            Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: self.0.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
                funds: vec![],
            })),
        )
    }
}
