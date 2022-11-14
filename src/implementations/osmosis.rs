use crate::{Burn, CwTokenResponse, CwTokenResult, Instantiate, Mint, Receive, VaultToken};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdError, StdResult, Uint128,
};
use cw_asset::AssetInfo;
use osmosis_std::types::cosmos::bank::v1beta1::BankQuerier;
use osmosis_std::types::cosmos::base::v1beta1::Coin as CoinMsg;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{MsgBurn, MsgCreateDenom, MsgMint};
use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;

#[cw_serde]
/// Representation of a native token created using the Osmosis Token Factory.
/// The denom of the token will be `factory/{owner}/{subdenom}`. If this token
/// has not yet been created, the `instantiate` function must first be called
/// and its response included in the transaction. If the token has already been
/// created an [`OsmosisDenom`] object can be created directly
/// using [`OsmosisDenom::new`] or [`OsmosisDenom::from_native_denom`]. Note
/// that currently only the creator of the denom can mint or burn it.
///
/// This struct implements the [`VaultToken`] trait.
pub struct OsmosisDenom {
    /// Creator and owner of the denom. Only this address can mint and burn
    /// tokens.
    pub owner: String,
    /// The subdenom of the token. All tokens created using the token factory
    /// have the format `factory/{owner}/{subdenom}`.
    pub subdenom: String,
}

impl OsmosisDenom {
    /// Creates a new [`OsmosisDenom`] obj instance
    pub const fn new(owner: String, subdenom: String) -> Self {
        Self { owner, subdenom }
    }

    /// Create an [`OsmosisDenom`] from a string. `denom` must be the full denom
    /// of the token, in the format `factory/{owner}/{subdenom}`.
    ///
    /// ## Errors
    /// Will return [`StdError`] if `denom` does not follow the required format.
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
    /// Returns the full denom of the token, in the format
    /// `factory/{owner}/{subdenom}`.
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
        let bank_querier = BankQuerier::new(&deps.querier);
        let amount_str = bank_querier
            .supply_of(self.to_string())?
            .amount
            .map(|c| c.amount)
            .ok_or(StdError::not_found("amount in supply response"))?;
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
    fn receive(
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

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    use super::*;

    const SENDER: &str = "sender";
    const SUBDENOM: &str = "subdenom";

    #[test]
    fn test_to_string() {
        let denom = OsmosisDenom::new(SENDER.to_string(), SUBDENOM.to_string());
        assert_eq!(
            denom.to_string(),
            format!("factory/{}/{}", SENDER, SUBDENOM)
        );
    }

    #[test]
    fn test_from_native_denom() {
        // Valid denom
        let denom = OsmosisDenom::from_native_denom("factory/sender/subdenom").unwrap();
        assert_eq!(denom.owner, "sender");
        assert_eq!(denom.subdenom, "subdenom");

        // Denom contains too few parts
        assert!(OsmosisDenom::from_native_denom("factory/sender").is_err());

        // Denom contains too many parts
        assert!(OsmosisDenom::from_native_denom("factory/sender/subdenom/extra").is_err());

        // Denom does not start with "factory"
        assert!(OsmosisDenom::from_native_denom("wrong/sender/subdenom").is_err());
    }

    #[test]
    fn test_into_asset_info() {
        let denom = OsmosisDenom::new(SENDER.to_string(), SUBDENOM.to_string());
        let asset_info: AssetInfo = denom.into();
        assert_eq!(
            asset_info,
            AssetInfo::Native(format!("factory/{}/{}", SENDER, SUBDENOM))
        );
    }

    #[test]
    fn test_try_from_asset_info() {
        // Native asset
        let asset_info = AssetInfo::Native(format!("factory/{}/{}", SENDER, SUBDENOM));
        let denom = OsmosisDenom::try_from(asset_info).unwrap();
        assert_eq!(denom.owner, SENDER);
        assert_eq!(denom.subdenom, SUBDENOM);

        // Non-native asset
        let asset_info = AssetInfo::Cw20(Addr::unchecked("addr"));
        let err = OsmosisDenom::try_from(asset_info).unwrap_err();

        assert_eq!(
            err,
            StdError::generic_err("Cannot convert non-native asset to OsmosisDenom.")
        );
    }

    #[test]
    fn test_receive_vault_token() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let denom = OsmosisDenom::new(env.contract.address.to_string(), SUBDENOM.to_string());

        // Set up MessageInfo with funds
        let sent_coin = Coin {
            denom: denom.to_string(),
            amount: Uint128::from(1000u128),
        };
        let info = MessageInfo {
            sender: Addr::unchecked(SENDER),
            funds: vec![sent_coin.clone()],
        };

        // Try to receive more than was sent
        let mut receive_coin = Coin {
            denom: denom.to_string(),
            amount: Uint128::from(5000u128),
        };
        let err = denom
            .receive(deps.as_mut(), &env, &info, receive_coin.amount)
            .unwrap_err();

        // Assert error message
        assert_eq!(
            err,
            StdError::generic_err(format!("Expected to receive {}", receive_coin))
        );

        // Try to receive less than was sent
        receive_coin.amount = Uint128::from(500u128);
        let err = denom
            .receive(deps.as_mut(), &env, &info, receive_coin.amount)
            .unwrap_err();

        // Assert error message
        assert_eq!(
            err,
            StdError::generic_err(format!("Expected to receive {}", receive_coin))
        );

        // Try to receive exactly what was sent
        denom
            .receive(deps.as_mut(), &env, &info, sent_coin.amount)
            .unwrap();
    }
}
