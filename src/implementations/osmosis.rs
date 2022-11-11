use crate::{Burn, CwTokenResponse, CwTokenResult, Instantiate, Mint, Receive, VaultToken};
use apollo_proto_rust::cosmos::bank::v1beta1::{QuerySupplyOfRequest, QuerySupplyOfResponse};
use apollo_proto_rust::utils::encode;
use apollo_proto_rust::OsmosisTypeURLs;
use cosmwasm_schema::cw_serde;

use cosmwasm_std::{
    Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, QueryRequest,
    Response, StdError, StdResult, Uint128,
};
use cw_asset::AssetInfo;
use osmosis_std::types::cosmos::base::v1beta1::Coin as CoinMsg;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{MsgBurn, MsgCreateDenom, MsgMint};

use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;

#[cw_serde]
/// Handle Osmosis Denom path for factory on cw-token
pub struct OsmosisDenom {
    /// grouping name
    pub owner: String,
    /// sub denom
    pub subdenom: String,
}

impl OsmosisDenom {
    /// Creates a new [`OsmosisDenom`] obj instance
    pub const fn new(owner: String, subdenom: String) -> Self {
        Self { owner, subdenom }
    }

    /// Create a Denom using factory owner
    /// # Errors
    ///
    /// Will return `Err` if `denom` is not well formed.
    pub fn from_native_denom(denom: &str) -> StdResult<Self> {
        let parts: Vec<_> = denom.split('/').collect();

        if parts.len() != 3 || parts[0] != "factory" {
            return Err(StdError::generic_err(
                "Can't create OsmosisDenom from invalid denom.",
            ));
        }

        Ok(Self::new(parts[1].to_string(), parts[2].to_string()))
    }
}

impl Display for OsmosisDenom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "factory/{}/{}", self.owner, self.subdenom)
    }
}

impl From<OsmosisDenom> for AssetInfo {
    fn from(denom: OsmosisDenom) -> Self {
        Self::Native(denom.to_string())
    }
}

impl TryFrom<AssetInfo> for OsmosisDenom {
    type Error = StdError;

    fn try_from(asset_info: AssetInfo) -> StdResult<Self> {
        match asset_info {
            AssetInfo::Native(denom) => Self::from_native_denom(denom.as_str()),
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

impl VaultToken for OsmosisDenom {
    fn query_balance<A: Into<String>>(&self, deps: Deps, address: A) -> CwTokenResult<Uint128> {
        Ok(deps
            .querier
            .query_balance(address, self.to_string())?
            .amount)
    }

    fn query_total_supply(&self, deps: Deps) -> CwTokenResult<Uint128> {
        let amount_str = deps
            .querier
            .query::<QuerySupplyOfResponse>(&QueryRequest::Stargate {
                path: OsmosisTypeURLs::QuerySupplyOf.to_string(),
                data: encode(QuerySupplyOfRequest {
                    denom: self.to_string(),
                }),
            })?
            .amount
            .map(|c| c.amount)
            .ok_or_else(|| StdError::generic_err("No amount in supply response."))?;

        Ok(Uint128::from_str(&amount_str)?)
    }
}

impl Mint for OsmosisDenom {
    fn mint(
        &self,
        _deps: DepsMut,
        env: &Env,
        recipient: &Addr,
        amount: Uint128,
    ) -> CwTokenResponse {
        let mint_msg: CosmosMsg = (MsgMint {
            amount: Some(CoinMsg {
                denom: self.to_string(),
                amount: amount.to_string(),
            }),
            sender: env.contract.address.to_string(),
        })
        .into();

        Ok(Response::new().add_messages(vec![
            mint_msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.to_string(),
                amount: vec![Coin {
                    denom: self.to_string(),
                    amount,
                }],
            }),
        ]))
    }
}

impl Burn for OsmosisDenom {
    fn burn(&self, _deps: DepsMut, env: &Env, amount: Uint128) -> CwTokenResponse {
        Ok(Response::new().add_message(MsgBurn {
            amount: Some(CoinMsg {
                denom: self.to_string(),
                amount: amount.to_string(),
            }),
            sender: env.contract.address.to_string(),
        }))
    }
}

impl Instantiate for OsmosisDenom {
    fn instantiate(&self, _deps: DepsMut, _init_info: Option<Binary>) -> CwTokenResponse {
        let init_msg: CosmosMsg = (MsgCreateDenom {
            sender: self.owner.clone(),
            subdenom: self.subdenom.clone(),
        })
        .into();

        let init_event =
            Event::new("apollo/cw-token/instantiate").add_attribute("denom", self.to_string());
        Ok(Response::new().add_message(init_msg).add_event(init_event))
    }
}

impl Receive for OsmosisDenom {
    fn receive_vault_token(
        &self,
        _deps: DepsMut,
        _env: &Env,
        info: &MessageInfo,
        amount: Uint128,
    ) -> StdResult<()> {
        let required = Coin {
            denom: self.to_string(),
            amount,
        };
        if !info.funds.contains(&required) {
            return Err(StdError::generic_err(format!(
                "Expected to receive {}",
                required
            )));
        }
        Ok(())
    }
}
