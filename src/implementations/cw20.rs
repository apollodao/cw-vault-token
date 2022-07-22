use cosmwasm_std::{
    to_binary, Addr, Api, CosmosMsg, QueryRequest, Reply, Response, StdError, StdResult, Storage,
    SubMsg, SubMsgResponse, Uint128, WasmMsg, WasmQuery,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use cw_asset::AssetInfo;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt::Display};

use crate::{
    token::{Burn, Instantiate, Mint},
    utils::unwrap_reply,
    CwTokenError, Token, TransferFrom,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
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
            AssetInfo::Cw1155(_, _) => Err(StdError::generic_err(
                "Cannot convert Cw1155 asset to Cw20.",
            )),
        }
    }
}

// ------ Implement Instantiate for Cw20Asset ------

pub const REPLY_SAVE_CW20_ADDRESS: u64 = 14509;

fn parse_contract_addr_from_instantiate_event(
    api: &dyn Api,
    response: SubMsgResponse,
) -> StdResult<Addr> {
    let event = response
        .events
        .iter()
        .find(|event| event.ty == "instantiate")
        .ok_or_else(|| StdError::generic_err("cannot find `instantiate` event"))?;

    let contract_addr_str = &event
        .attributes
        .iter()
        .find(|attr| attr.key == "_contract_address")
        .ok_or_else(|| StdError::generic_err("cannot find `_contract_address` attribute"))?
        .value;

    api.addr_validate(contract_addr_str)
}

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw20Instantiator {
    pub label: String,
    pub admin: Option<String>,
    pub code_id: u64,
    pub cw20_init_msg: Cw20InstantiateMsg,
}

impl Instantiate<Cw20> for Cw20Instantiator {
    fn instantiate_msg(&self) -> StdResult<SubMsg> {
        Ok(SubMsg::reply_always(
            WasmMsg::Instantiate {
                admin: self.admin.clone(),
                code_id: self.code_id,
                msg: to_binary(&self.cw20_init_msg)?,
                funds: vec![],
                label: self.label.clone(),
            },
            REPLY_SAVE_CW20_ADDRESS,
        ))
    }

    fn save_asset(
        storage: &mut dyn Storage,
        api: &dyn Api,
        reply: &Reply,
        item: Item<Cw20>,
    ) -> Result<Response, CwTokenError> {
        match reply.id {
            REPLY_SAVE_CW20_ADDRESS => {
                let res = unwrap_reply(reply)?;
                let addr = parse_contract_addr_from_instantiate_event(api, res)?;

                item.save(storage, &Cw20(addr.clone()))?;

                Ok(Response::new()
                    .add_attribute("action", "save_osmosis_denom")
                    .add_attribute("addr", &addr))
            }
            _ => Err(CwTokenError::InvalidReplyId {}),
        }
    }
}
