use crate::{
    token::{Burn, Mint},
    CwTokenError, CwTokenResponse, CwTokenResult, Instantiate, Send, Token, TokenStorage,
    TransferFrom,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    from_binary, to_binary, wasm_execute, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, Event,
    MessageInfo, QueryRequest, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
    WasmQuery,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_asset::AssetInfo;
use cw_utils::parse_reply_instantiate_data;
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

#[cw_serde]
pub struct Cw20InitInfo {
    code_id: u64,
    admin: Option<String>,
    funds: Vec<Coin>,
    label: String,
    init_msg: Binary,
}

/// We implement default so that you can call Cw20::default().instantiate(...)
impl Default for Cw20 {
    fn default() -> Self {
        Self(Addr::unchecked(String::default()))
    }
}

pub const REPLY_SAVE_CW20_ADDRESS: u64 = 14509;

impl Instantiate for Cw20 {
    fn instantiate(_deps: DepsMut, _env: &Env, init_info: Option<Binary>) -> CwTokenResponse {
        let msg: Cw20InitInfo =
            from_binary(&init_info.ok_or(StdError::generic_err("init_info requried for Cw20"))?)?;

        let init_msg = SubMsg::reply_always(
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: msg.admin,
                code_id: msg.code_id,
                msg: msg.init_msg,
                funds: msg.funds,
                label: msg.label.clone(),
            }),
            REPLY_SAVE_CW20_ADDRESS,
        );
        let init_event = Event::new("create_cw20").add_attribute("label", msg.label);

        Ok(Response::new()
            .add_submessage(init_msg)
            .add_event(init_event))
    }

    fn reply_save_token(deps: DepsMut, _env: &Env, reply: &Reply) -> CwTokenResponse {
        match reply.id {
            REPLY_SAVE_CW20_ADDRESS => {
                let res = parse_reply_instantiate_data(reply.clone())?;

                let addr = deps.api.addr_validate(&res.contract_address)?;

                Cw20(addr.to_owned()).save(deps.storage)?;

                Ok(Response::new()
                    .add_attribute("action", "save_cw20_addr")
                    .add_attribute("contract_addr", &addr))
            }
            _ => Err(CwTokenError::InvalidReplyId {}),
        }
    }
}

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

impl Send for Cw20 {
    fn send<A: Into<String>>(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        contract: A,
        amount: Uint128,
        msg: Binary,
    ) -> CwTokenResponse {
        Ok(Response::new().add_message(wasm_execute(
            self.0.to_string(),
            &Cw20ExecuteMsg::Send {
                contract: contract.into(),
                amount,
                msg,
            },
            vec![],
        )?))
    }

    fn send_from<A: Into<String>>(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        owner: A,
        contract: A,
        amount: Uint128,
        msg: Binary,
    ) -> CwTokenResponse {
        Ok(Response::new().add_message(wasm_execute(
            self.0.to_string(),
            &Cw20ExecuteMsg::SendFrom {
                owner: owner.into(),
                contract: contract.into(),
                amount,
                msg,
            },
            vec![],
        )?))
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

impl TokenStorage for Cw20 {}
