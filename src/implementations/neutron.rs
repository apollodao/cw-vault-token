use crate::{Burn, CwTokenResponse, CwTokenResult, Instantiate, Mint, Receive, VaultToken};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    attr, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdError, StdResult, Uint128,
};
use cw_storage_plus::Item;
use osmosis_std::types::cosmos::base::v1beta1::Coin as CoinMsg;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::{MsgBurn, MsgCreateDenom, MsgMint};
use std::fmt::Display;

/// Item that stores the total supply of the denom. This is needed because a supply
/// query requires either CosmWasm 1.1 or a stargate query, neither of which are
/// currently supported by Neutron.
pub const TOTAL_SUPPLY: Item<Uint128> = Item::new("neutron_denom_total_supply");

/// Representation of a native token created using the Neutron Token Factory.
/// The denom of the token will be `factory/{owner}/{subdenom}`. If this token
/// has not yet been created, the `instantiate` function must first be called
/// and its response included in the transaction. If the token has already been
/// created an [`NeutronDenom`] object can be created directly
/// using [`NeutronDenom::new`] or [`NeutronDenom::from_native_denom`]. Note
/// that currently only the creator of the denom can mint or burn it.
///
/// This struct implements the [`VaultToken`] trait.
#[cw_serde]
pub struct NeutronDenom {
    /// Creator and owner of the denom. Only this address can mint and burn
    /// tokens.
    pub owner: String,
    /// The subdenom of the token. All tokens created using the token factory
    /// have the format `factory/{owner}/{subdenom}`.
    pub subdenom: String,
}

impl NeutronDenom {
    /// Creates a new [`NeutronDenom`] obj instance
    pub fn new(owner: String, subdenom: String) -> Self {
        Self { owner, subdenom }
    }

    /// Create an [`NeutronDenom`] from a string. `denom` must be the full denom
    /// of the token, in the format `factory/{owner}/{subdenom}`.
    ///
    /// ## Errors
    /// Will return [`StdError`] if `denom` does not follow the required format.
    pub fn from_native_denom(denom: &str) -> StdResult<Self> {
        let parts: Vec<_> = denom.split('/').collect();

        if parts.len() != 3 || parts[0] != "factory" {
            return Err(StdError::generic_err(
                "Can't create NeutronDenom from invalid denom.",
            ));
        }

        Ok(Self::new(parts[1].to_string(), parts[2].to_string()))
    }
}

impl Display for NeutronDenom {
    /// Returns the full denom of the token, in the format
    /// `factory/{owner}/{subdenom}`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "factory/{}/{}", self.owner, self.subdenom)
    }
}

impl VaultToken for NeutronDenom {
    fn query_balance<A: Into<String>>(&self, deps: Deps, address: A) -> CwTokenResult<Uint128> {
        Ok(deps
            .querier
            .query_balance(address, self.to_string())?
            .amount)
    }

    fn query_total_supply(&self, deps: Deps) -> CwTokenResult<Uint128> {
        Ok(TOTAL_SUPPLY.may_load(deps.storage)?.unwrap_or_default())
    }
}

impl Mint for NeutronDenom {
    fn mint(&self, deps: DepsMut, env: &Env, recipient: &Addr, amount: Uint128) -> CwTokenResponse {
        let mint_msg: CosmosMsg = (MsgMint {
            amount: Some(CoinMsg {
                denom: self.to_string(),
                amount: amount.to_string(),
            }),
            sender: env.contract.address.to_string(),
        })
        .into();

        let total_supply = TOTAL_SUPPLY.may_load(deps.storage)?.unwrap_or_default();
        TOTAL_SUPPLY.save(deps.storage, &total_supply.checked_add(amount)?)?;

        let event = Event::new("apollo/cw-vault-token/neutron").add_attributes(vec![
            attr("action", "mint"),
            attr("denom", self.to_string()),
            attr("amount", amount.to_string()),
            attr("recipient", recipient.to_string()),
        ]);

        Ok(Response::new()
            .add_messages(vec![
                mint_msg,
                CosmosMsg::Bank(BankMsg::Send {
                    to_address: recipient.to_string(),
                    amount: vec![Coin {
                        denom: self.to_string(),
                        amount,
                    }],
                }),
            ])
            .add_event(event))
    }
}

impl Burn for NeutronDenom {
    fn burn(&self, deps: DepsMut, env: &Env, amount: Uint128) -> CwTokenResponse {
        let total_supply = TOTAL_SUPPLY.may_load(deps.storage)?.unwrap_or_default();
        TOTAL_SUPPLY.save(deps.storage, &total_supply.checked_sub(amount)?)?;

        let event = Event::new("apollo/cw-vault-token/neutron").add_attributes(vec![
            attr("action", "burn"),
            attr("denom", self.to_string()),
            attr("amount", amount.to_string()),
        ]);
        Ok(Response::new()
            .add_message(MsgBurn {
                amount: Some(CoinMsg {
                    denom: self.to_string(),
                    amount: amount.to_string(),
                }),
                sender: env.contract.address.to_string(),
            })
            .add_event(event))
    }
}

impl Instantiate for NeutronDenom {
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

impl Receive for NeutronDenom {
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
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        SubMsg,
    };

    use test_case::test_case;

    use super::*;

    const SENDER: &str = "sender";
    const SUBDENOM: &str = "subdenom";

    #[test]
    fn to_string() {
        let denom = NeutronDenom::new(SENDER.to_string(), SUBDENOM.to_string());
        assert_eq!(
            denom.to_string(),
            format!("factory/{}/{}", SENDER, SUBDENOM)
        );
    }

    #[test_case("factory/sender/subdenom" ; "valid denom")]
    #[test_case("factory/sender" => panics ; "denom contains too few parts")]
    #[test_case("factory/sender/subdenom/extra" => panics ; "denom contains too many parts")]
    #[test_case("wrong/sender/subdenom" => panics ; "denom does not start with \"factory\"")]
    fn from_native_denom(denom: &str) {
        // Valid denom
        let denom = NeutronDenom::from_native_denom(denom).unwrap();
        assert_eq!(denom.owner, "sender");
        assert_eq!(denom.subdenom, "subdenom");
    }

    #[test_case(Uint128::from(1000u128), Uint128::from(1000u128) ; "sent amount correct")]
    #[test_case(Uint128::from(1000u128), Uint128::from(1001u128) => panics ; "sent amount too large")]
    #[test_case(Uint128::from(1000u128), Uint128::from(999u128) => panics ; "sent amount too small")]
    fn test_receive_vault_token(recieve_amount: Uint128, sent_amount: Uint128) {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let denom = NeutronDenom::new(env.contract.address.to_string(), SUBDENOM.to_string());

        // Set up MessageInfo with funds
        let sent_coin = Coin {
            denom: denom.to_string(),
            amount: sent_amount,
        };
        let info = MessageInfo {
            sender: Addr::unchecked(SENDER),
            funds: vec![sent_coin],
        };

        // Try to receive more than was sent
        let receive_coin = Coin {
            denom: denom.to_string(),
            amount: recieve_amount,
        };
        denom
            .receive(deps.as_mut(), &env, &info, receive_coin.amount)
            .unwrap();
    }

    #[test]
    fn test_mint_and_burn() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let recipient = Addr::unchecked("recipient");
        let mint_amount = Uint128::from(1000u128);
        let burn_amount = Uint128::from(500u128);

        let denom = NeutronDenom::new(env.contract.address.to_string(), SUBDENOM.to_string());

        // Assert zero supply at start
        let total_supply = denom.query_total_supply(deps.as_ref()).unwrap();
        assert_eq!(total_supply, Uint128::zero());

        // Mint tokens
        let res = denom
            .mint(deps.as_mut(), &env, &recipient, mint_amount)
            .unwrap();

        // Assert correct messages in response
        let msgs = res.messages;
        assert_eq!(msgs.len(), 2);
        let mint_msg = &msgs[0];
        let bank_msg = &msgs[1];
        let mint_cosmos_msg: CosmosMsg = MsgMint {
            amount: Some(CoinMsg {
                denom: denom.to_string(),
                amount: mint_amount.to_string(),
            }),
            sender: env.contract.address.to_string(),
        }
        .into();
        assert_eq!(mint_msg, &SubMsg::new(mint_cosmos_msg));
        assert_eq!(
            bank_msg,
            &SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.to_string(),
                amount: vec![Coin {
                    denom: denom.to_string(),
                    amount: mint_amount,
                }],
            }))
        );

        // Assert total supply updated
        let total_supply = denom.query_total_supply(deps.as_ref()).unwrap();
        assert_eq!(total_supply, mint_amount);

        // Burn tokens
        let res = denom.burn(deps.as_mut(), &env, burn_amount).unwrap();

        // Assert correct messages in response
        let msgs = res.messages;
        assert_eq!(msgs.len(), 1);
        let burn_msg = &msgs[0];
        let burn_cosmos_msg: CosmosMsg = MsgBurn {
            amount: Some(CoinMsg {
                denom: denom.to_string(),
                amount: burn_amount.to_string(),
            }),
            sender: env.contract.address.to_string(),
        }
        .into();
        assert_eq!(burn_msg, &SubMsg::new(burn_cosmos_msg));

        // Assert total supply updated
        let total_supply = denom.query_total_supply(deps.as_ref()).unwrap();
        assert_eq!(total_supply, mint_amount - burn_amount);
    }
}
