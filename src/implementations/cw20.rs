use crate::{
    token::{Burn, Mint},
    CwTokenResponse, CwTokenResult, Instantiate, Token, TransferFrom,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest,
    Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw20_base::msg::InstantiateMsg;
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

impl Instantiate for Cw20 {
    fn instantiate(&self, _deps: DepsMut, init_info: Binary) -> CwTokenResponse {
        let _msg: InstantiateMsg = from_binary(&init_info)?;

        //TODO: Where to store codeid?
        // Ok(Response::new().add_message(wasm_instantiate(code_id, msg, vec![], msg.name)))
        Ok(Response::default())
    }

    fn save_token(
        _deps: DepsMut,
        _env: &Env,
        _reply: &cosmwasm_std::Reply,
        _item: &cw_storage_plus::Item<Self>,
    ) -> CwTokenResponse {
        todo!()
    }
}

pub const REPLY_SAVE_CW20_ADDRESS: u64 = 14509;

impl Token for Cw20 {
    fn transfer<A: Into<String>>(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        recipient: A,
        amount: Uint128,
    ) -> CwTokenResponse {
        Ok(
            Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: self.0.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: recipient.into(),
                    amount,
                })?,
                funds: vec![],
            })),
        )
    }

    fn query_balance<A: Into<String>>(&self, deps: Deps, address: A) -> CwTokenResult<Uint128> {
        Ok(deps
            .querier
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
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        from: A,
        to: B,
        amount: Uint128,
    ) -> CwTokenResponse {
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
        _deps: DepsMut,
        _sender: A,
        recipient: B,
        amount: Uint128,
    ) -> CwTokenResponse {
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
    fn burn<A: Into<String>>(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _sender: A,
        amount: Uint128,
    ) -> CwTokenResponse {
        Ok(
            Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: self.0.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
                funds: vec![],
            })),
        )
    }
}
