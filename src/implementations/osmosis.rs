use crate::token::{Burn, Instantiate, Mint};
use crate::{CwTokenResponse, CwTokenResult, Token, TokenStorage};
use apollo_proto_rust::cosmos::base::v1beta1::Coin as CoinMsg;
use apollo_proto_rust::osmosis::tokenfactory::v1beta1::{MsgBurn, MsgCreateDenom, MsgMint};
use apollo_proto_rust::utils::encode;
use apollo_proto_rust::OsmosisTypeURLs;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    from_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo,
    Response, StdError, StdResult, Uint128,
};
use cw_asset::AssetInfo;

use std::convert::TryFrom;
use std::fmt::Display;

#[cw_serde]
pub struct OsmosisDenom {
    pub owner: String,
    pub subdenom: String,
}

impl OsmosisDenom {
    pub fn new(owner: String, subdenom: String) -> Self {
        OsmosisDenom { owner, subdenom }
    }

    pub fn from_native_denom(denom: String) -> StdResult<Self> {
        let parts: Vec<_> = denom.split("/").collect();

        if parts.len() != 3 || parts[0] != "factory" {
            return Err(StdError::generic_err(
                "Can't create OsmosisDenom from invalid denom.",
            ));
        }

        Ok(OsmosisDenom::new(
            parts[1].to_string(),
            parts[2].to_string(),
        ))
    }
}

impl Display for OsmosisDenom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "factory/{}/{}", self.owner, self.subdenom)
    }
}

impl From<OsmosisDenom> for AssetInfo {
    fn from(denom: OsmosisDenom) -> Self {
        AssetInfo::Native(denom.to_string())
    }
}

impl TryFrom<AssetInfo> for OsmosisDenom {
    type Error = StdError;

    fn try_from(asset_info: AssetInfo) -> StdResult<Self> {
        match asset_info {
            AssetInfo::Native(denom) => OsmosisDenom::from_native_denom(denom),
            _ => Err(StdError::generic_err(
                "Cannot convert non-native asset to OsmosisDenom.",
            )),
        }
    }
}

impl TryFrom<&AssetInfo> for OsmosisDenom {
    type Error = StdError;

    fn try_from(asset_info: &AssetInfo) -> StdResult<Self> {
        Self::try_from(asset_info.clone())
    }
}

impl Token for OsmosisDenom {
    fn transfer<A: Into<String>>(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        to: A,
        amount: Uint128,
    ) -> CwTokenResponse {
        Ok(Response::new().add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: to.into(),
            amount: vec![Coin {
                denom: self.to_string(),
                amount,
            }],
        })))
    }

    fn query_balance<A: Into<String>>(&self, deps: Deps, address: A) -> CwTokenResult<Uint128> {
        Ok(deps
            .querier
            .query_balance(address, self.to_string())?
            .amount)
    }

    fn is_native() -> bool {
        true
    }
}

impl Mint for OsmosisDenom {
    fn mint<A: Into<String>, B: Into<String>>(
        &self,
        _deps: DepsMut,
        sender: A,
        recipient: B,
        amount: Uint128,
    ) -> CwTokenResponse {
        Ok(Response::new().add_messages(vec![
            CosmosMsg::Stargate {
                type_url: OsmosisTypeURLs::Mint.to_string(),
                value: encode(MsgMint {
                    amount: Some(CoinMsg {
                        denom: self.to_string(),
                        amount: amount.to_string(),
                    }),
                    sender: sender.into(),
                }),
            },
            CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.into(),
                amount: vec![Coin {
                    denom: self.to_string(),
                    amount,
                }],
            }),
        ]))
    }
}

impl Burn for OsmosisDenom {
    fn burn<A: Into<String>>(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        sender: A,
        amount: Uint128,
    ) -> CwTokenResponse {
        Ok(Response::new().add_message(CosmosMsg::Stargate {
            type_url: OsmosisTypeURLs::Burn.to_string(),
            value: encode(MsgBurn {
                amount: Some(CoinMsg {
                    denom: self.to_string(),
                    amount: amount.to_string(),
                }),
                sender: sender.into(),
            }),
        }))
    }
}

#[cw_serde]
pub struct OsmosisDenomInitInfo {
    pub symbol: String,
}

impl Instantiate for OsmosisDenom {
    fn instantiate(deps: DepsMut, env: &Env, init_info: Option<Binary>) -> CwTokenResponse {
        let msg: OsmosisDenomInitInfo = from_binary(
            &init_info.ok_or(StdError::generic_err("init_info required for OsmosisDenom"))?,
        )?;

        let osmosis_denom = OsmosisDenom {
            owner: env.contract.address.to_string(),
            subdenom: msg.symbol,
        };

        // create denom with Osmosis Token Factory
        let init_msg = CosmosMsg::Stargate {
            type_url: OsmosisTypeURLs::CreateDenom.to_string(),
            value: encode(MsgCreateDenom {
                sender: osmosis_denom.owner.to_string(),
                subdenom: osmosis_denom.subdenom.to_string(),
            }),
        };

        // Store denom in contract state
        osmosis_denom.save(deps.storage)?;

        let init_event = Event::new("apollo/cw-token/instantiate")
            .add_attribute("denom", osmosis_denom.to_string());
        Ok(Response::new().add_message(init_msg).add_event(init_event))
    }
}

impl TokenStorage for OsmosisDenom {}
