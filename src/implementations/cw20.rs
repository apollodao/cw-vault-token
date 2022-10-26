use crate::{
    token::{Burn, Mint},
    CwTokenError, CwTokenResponse, CwTokenResult, Instantiate, Token,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo,
    QueryRequest, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_asset::AssetInfo;
use cw_storage_plus::Item;
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

pub const REPLY_SAVE_CW20_ADDRESS: u64 = 14509;

#[cw_serde]
pub struct Cw20InitInfo {
    pub code_id: u64,
    pub admin: Option<String>,
    pub funds: Vec<Coin>,
    pub label: String,
    pub init_msg: Binary,
}

/// We implement default so that you can call Cw20::default().instantiate(...)
impl Default for Cw20 {
    fn default() -> Self {
        Self(Addr::unchecked(String::default()))
    }
}

impl Cw20 {
    /// Saves the token to the storage in the provided `item`. This function should
    /// be called in the `reply` entry point of the contract after `Self::instantiate`
    /// has been called in the `instantiate` entry point.
    ///
    /// Arguments:
    /// - reply: The reply received to the `reply` entry point.
    /// - item: The `Item` to which the token should be saved.
    ///
    /// Returns a Response containing the messages to save the instantiated token.
    ///
    /// This is needed because as opposed to OsmosisDenom and Cw4626, when
    /// instantiating a Cw20 we don't know the address until after we receive a reply.
    ///
    /// ## Example
    /// ```
    /// #[cfg_attr(not(feature = "library"), entry_point)]
    /// pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    ///     MyToken::save_token(deps, env, reply)
    /// }
    /// ```
    pub fn save_token(
        deps: DepsMut,
        _env: &Env,
        reply: &Reply,
        item: &Item<Self>,
    ) -> CwTokenResponse {
        match reply.id {
            REPLY_SAVE_CW20_ADDRESS => {
                let res = parse_reply_instantiate_data(reply.clone())?;

                let addr = deps.api.addr_validate(&res.contract_address)?;

                item.save(deps.storage, &Self(addr.clone()))?;

                Ok(Response::new()
                    .add_attribute("action", "save_cw20_addr")
                    .add_attribute("contract_addr", &addr))
            }
            _ => Err(CwTokenError::InvalidReplyId {}),
        }
    }
}

impl Instantiate for Cw20 {
    fn instantiate(&self, _deps: DepsMut, init_info: Option<Binary>) -> CwTokenResponse {
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

impl Mint for Cw20 {
    fn mint(
        &self,
        _deps: DepsMut,
        _env: &Env,
        recipient: &Addr,
        amount: Uint128,
    ) -> CwTokenResponse {
        Ok(
            Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: self.0.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: recipient.to_string(),
                    amount,
                })?,
                funds: vec![],
            })),
        )
    }
}

impl Burn for Cw20 {
    fn burn(
        &self,
        _deps: DepsMut,
        _env: &Env,
        _info: &MessageInfo,
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
