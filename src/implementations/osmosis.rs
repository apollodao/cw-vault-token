use crate::token::{Burn, Instantiate, Mint};
use crate::utils::unwrap_reply;
use crate::{CwTokenError, Token};
use apollo_proto_rust::cosmos::base::v1beta1::Coin as CoinMsg;
use apollo_proto_rust::osmosis::tokenfactory::v1beta1::{MsgBurn, MsgCreateDenom, MsgMint};
use apollo_proto_rust::utils::encode;
use apollo_proto_rust::OsmosisTypeURLs;
use cosmwasm_std::{
    from_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, DepsMut, Env, Event, QuerierWrapper,
    Reply, Response, StdError, StdResult, SubMsg, SubMsgResponse, Uint128,
};
use cw_asset::AssetInfo;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct OsmosisDenom(pub String);

impl Display for OsmosisDenom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<OsmosisDenom> for AssetInfo {
    fn from(denom: OsmosisDenom) -> Self {
        AssetInfo::Native(denom.0)
    }
}

impl TryFrom<AssetInfo> for OsmosisDenom {
    type Error = StdError;

    fn try_from(asset_info: AssetInfo) -> StdResult<Self> {
        match asset_info {
            AssetInfo::Cw20(_) => Err(StdError::generic_err(
                "Cannot convert Cw20 asset to OsmosisDenom.",
            )),
            AssetInfo::Native(denom) => {
                let parts: Vec<&str> = denom.split('/').collect();
                if parts.len() != 3 || parts[0] != "factory" {
                    return Err(StdError::generic_err("Invalid denom for OsmosisDenom."));
                }
                Ok(OsmosisDenom(denom))
            }
            AssetInfo::Cw1155(_, _) => Err(StdError::generic_err(
                "Cannot convert Cw1155 asset to OsmosisDenom.",
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
    fn transfer<A: Into<String>>(&self, to: A, amount: Uint128) -> StdResult<Response> {
        Ok(Response::new().add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: to.into(),
            amount: vec![Coin {
                denom: self.0.clone(),
                amount,
            }],
        })))
    }

    fn query_balance<A: Into<String>>(
        &self,
        querier: &QuerierWrapper,
        address: A,
    ) -> StdResult<Uint128> {
        Ok(querier.query_balance(address, self.0.clone())?.amount)
    }

    fn is_native() -> bool {
        true
    }
}

impl Mint for OsmosisDenom {
    fn mint<A: Into<String>, B: Into<String>>(
        &self,
        sender: A,
        recipient: B,
        amount: Uint128,
    ) -> StdResult<Response> {
        Ok(Response::new().add_messages(vec![
            CosmosMsg::Stargate {
                type_url: OsmosisTypeURLs::Mint.to_string(),
                value: encode(MsgMint {
                    amount: Some(CoinMsg {
                        denom: self.0.clone(),
                        amount: amount.to_string(),
                    }),
                    sender: sender.into(),
                }),
            },
            CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.into(),
                amount: vec![Coin {
                    denom: self.0.clone(),
                    amount,
                }],
            }),
        ]))
    }
}

impl Burn for OsmosisDenom {
    fn burn<A: Into<String>>(&self, sender: A, amount: Uint128) -> StdResult<Response> {
        Ok(Response::new().add_message(CosmosMsg::Stargate {
            type_url: OsmosisTypeURLs::Burn.to_string(),
            value: encode(MsgBurn {
                amount: Some(CoinMsg {
                    denom: self.0.clone(),
                    amount: amount.to_string(),
                }),
                sender: sender.into(),
            }),
        }))
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct OsmosisDenomInfo {
    pub denom: String,
    pub sender: String,
}

pub const REPLY_SAVE_OSMOSIS_DENOM: u64 = 14508;

fn parse_osmosis_denom_from_instantiate_event(response: SubMsgResponse) -> StdResult<String> {
    let event = response
        .events
        .iter()
        .find(|event| event.ty == "create_denom")
        .ok_or_else(|| StdError::generic_err("cannot find `create_denom` event"))?;

    let denom = &event
        .attributes
        .iter()
        .find(|attr| attr.key == "new_token_denom")
        .ok_or_else(|| StdError::generic_err("cannot find `new_token_denom` attribute"))?
        .value;

    Ok(denom.to_string())
}

impl Instantiate for OsmosisDenom {
    fn instantiate(&self, init_info: Binary) -> StdResult<Response> {
        // Deserialize the init info binary
        let init_info: OsmosisDenomInfo = from_binary(&init_info).map_err(|e| {
            StdError::generic_err(format!("failed to deserialize init info: {}", e))
        })?;

        let init_msg = SubMsg::reply_always(
            CosmosMsg::Stargate {
                type_url: OsmosisTypeURLs::CreateDenom.to_string(),
                value: encode(MsgCreateDenom {
                    sender: init_info.sender.to_string(),
                    subdenom: init_info.denom.to_string(),
                }),
            },
            REPLY_SAVE_OSMOSIS_DENOM,
        );
        let denom = format!("factory/{}/{}", init_info.sender, init_info.denom);
        let init_event = Event::new("create_denom").add_attribute("new_token_denom", denom);
        Ok(Response::new()
            .add_submessage(init_msg)
            .add_event(init_event))
    }

    fn save_token(
        deps: DepsMut,
        env: &Env,
        reply: &Reply,
        item: Item<OsmosisDenom>,
    ) -> Result<Response, CwTokenError> {
        match reply.id {
            REPLY_SAVE_OSMOSIS_DENOM => {
                let res = unwrap_reply(reply)?;
                let denom = parse_osmosis_denom_from_instantiate_event(res)
                    .map_err(|e| StdError::generic_err(format!("{}", e)))?;

                item.save(deps.storage, &OsmosisDenom(denom.clone()))?;

                Ok(Response::new()
                    .add_attribute("action", "save_osmosis_denom")
                    .add_attribute("denom", &denom))
            }
            _ => Err(CwTokenError::InvalidReplyId {}),
        }
    }
}

// TODO:
// * Verify owner function on OsmosisDenom
// * More useful functions?
// * Implement queries as trait
