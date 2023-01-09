use std::fmt::Display;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    attr, from_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw20::MarketingInfoResponse;
use cw20_base::contract::query_balance;
use cw20_base::msg::{InstantiateMarketingInfo, InstantiateMsg};
use cw20_base::state::{TokenInfo, BALANCES, MARKETING_INFO, TOKEN_INFO};
use cw20_base::ContractError;

use crate::{Burn, CwTokenResponse, CwTokenResult, Instantiate, Mint, Receive, VaultToken};

#[cw_serde]
/// Representation of a tokenized vault following the standard defined in
/// <https://github.com/apollodao/cosmwasm-vault-standard#cw4626>, and any
/// contract using this abstraction must implement the messages
/// defined in the standard. Note that `Cw4626` does not support the Cw20
/// Minter extension, so only the `cw4626` contract itself can mint tokens.
/// This implementation also does not support initial balances.
///
/// To keep compatibility with OsmosisDenom `burn_from` is not implemented.
/// This means that before tokens can be burned they must be transferred to
/// the `cw4626` contract using [`Cw4626::receive`].
///
/// This struct implements the [`VaultToken`] trait.
pub struct Cw4626 {
    address: Addr,
}

impl Cw4626 {
    /// Creates a new [`Cw4626`] instance from a reference to
    /// [`cosmwasm_std::Env`].
    ///
    /// ## Example
    /// Create a new [`Cw4626`] instance for the current contract and
    /// instantiate it.
    ///
    /// ```ignore
    /// pub fn instantiate(
    ///     deps: DepsMut,
    ///     env: Env,
    ///     info: MessageInfo,
    ///     msg: InstantiateMsg,
    /// ) -> Result<Response, ContractError> {
    ///     let cw4626 = Cw4626::new(&env);
    ///     cw4626.instantiate(deps, to_binary(&msg.init_info)?)
    /// }
    /// ```
    pub fn new(env: &Env) -> Self {
        Cw4626 {
            address: env.contract.address.clone(),
        }
    }
}

impl Display for Cw4626 {
    /// Returns the address of the contract as a string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.address)
    }
}

impl VaultToken for Cw4626 {
    fn query_balance<A: Into<String>>(&self, deps: Deps, address: A) -> CwTokenResult<Uint128> {
        Ok(query_balance(deps, address.into())?.balance)
    }

    fn query_total_supply(&self, deps: Deps) -> CwTokenResult<Uint128> {
        Ok(TOKEN_INFO.load(deps.storage)?.total_supply)
    }
}

impl Mint for Cw4626 {
    fn mint(
        &self,
        deps: DepsMut,
        _env: &Env,
        recipient: &Addr,
        amount: Uint128,
    ) -> CwTokenResponse {
        // Here we must copy-paste the code from cw20_base, because cw20 base does not
        // allow anyone to mint, and here we want anyone to be able to mint as long as
        // they deposit the correct depositable assets
        let recipient: String = recipient.to_string();

        if amount == Uint128::zero() {
            return Err(ContractError::InvalidZeroAmount {}.into());
        }

        let mut config = TOKEN_INFO
            .may_load(deps.storage)?
            .ok_or(ContractError::Unauthorized {})?;

        // update supply
        config.total_supply += amount;

        TOKEN_INFO.save(deps.storage, &config)?;

        // add amount to recipient balance
        let rcpt_addr = deps.api.addr_validate(&recipient)?;
        BALANCES.update(
            deps.storage,
            &rcpt_addr,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;

        let attrs = vec![
            attr("action", "mint"),
            attr("vault_token_address", self.to_string()),
            attr("amount", amount.to_string()),
            attr("recipient", recipient),
        ];
        let event = Event::new("apollo/cw-vault-token/cw4626").add_attributes(attrs.to_vec());

        let res = Response::new().add_event(event).add_attributes(attrs);
        Ok(res)
    }
}

impl Burn for Cw4626 {
    fn burn(&self, deps: DepsMut, env: &Env, amount: Uint128) -> CwTokenResponse {
        // lower balance
        BALANCES.update(
            deps.storage,
            &env.contract.address,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(amount)?)
            },
        )?;
        // reduce total_supply
        TOKEN_INFO.update(deps.storage, |mut meta| -> StdResult<_> {
            meta.total_supply = meta.total_supply.checked_sub(amount)?;
            Ok(meta)
        })?;

        let attrs = vec![
            attr("action", "burn"),
            attr("vault_token_address", self.to_string()),
            attr("amount", amount.to_string()),
        ];
        let event = Event::new("apollo/cw-vault-token/cw4626").add_attributes(attrs.to_vec());

        let res = Response::new().add_event(event).add_attributes(attrs);
        Ok(res)
    }
}

#[cw_serde]
/// Instantiate message for a cw4626 token. Contains the same fields as
/// [`cw20_base::msg::InstantiateMsg`], omitting `initial_balances` and
/// `minter`.
pub struct Cw4626InstantiateMsg {
    /// Name of the token
    pub name: String,
    /// Ticker symbol for the token
    pub symbol: String,
    /// Number of decimals
    pub decimals: u8,
    /// Optional marketing info
    pub marketing: Option<InstantiateMarketingInfo>,
}

impl From<Cw4626InstantiateMsg> for InstantiateMsg {
    fn from(msg: Cw4626InstantiateMsg) -> Self {
        InstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            decimals: msg.decimals,
            initial_balances: vec![],
            mint: None,
            marketing: msg.marketing,
        }
    }
}

impl Instantiate for Cw4626 {
    fn instantiate(&self, deps: DepsMut, init_info: Option<Binary>) -> CwTokenResponse {
        let msg: InstantiateMsg = from_binary::<Cw4626InstantiateMsg>(
            &init_info.ok_or_else(|| StdError::generic_err("init_info requried for Cw4626"))?,
        )?
        .into();

        // check valid token info
        msg.validate()?;

        // store token info
        let data = TokenInfo {
            name: msg.name,
            symbol: msg.symbol,
            decimals: msg.decimals,
            total_supply: Uint128::zero(),
            mint: None,
        };
        TOKEN_INFO.save(deps.storage, &data)?;

        if let Some(marketing) = msg.marketing {
            let data = MarketingInfoResponse {
                project: marketing.project,
                description: marketing.description,
                marketing: marketing
                    .marketing
                    .map(|addr| deps.api.addr_validate(&addr))
                    .transpose()?,
                logo: None,
            };
            MARKETING_INFO.save(deps.storage, &data)?;
        }

        let attrs = vec![
            attr("action", "instantiate"),
            attr("name", data.name),
            attr("symbol", data.symbol),
            attr("decimals", data.decimals.to_string()),
        ];
        let event = Event::new("apollo/cw-vault-token/cw4626").add_attributes(attrs.to_vec());

        Ok(Response::default().add_event(event).add_attributes(attrs))
    }
}

impl Receive for Cw4626 {
    /// Recieve the vault token from the caller's (info.sender) balance into the
    /// contract's balance.
    fn receive(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        amount: Uint128,
    ) -> StdResult<()> {
        let rcpt_addr = &env.contract.address;
        let owner_addr = &info.sender;

        BALANCES.update(
            deps.storage,
            owner_addr,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(amount)?)
            },
        )?;
        BALANCES.update(
            deps.storage,
            rcpt_addr,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier};
    use cosmwasm_std::{to_binary, MemoryStorage, OverflowError, OverflowOperation, OwnedDeps};

    use crate::CwTokenError;

    use super::*;

    const SENDER: &str = "sender";

    fn instantiate_cw4626(cw4626: Cw4626, deps: DepsMut) -> CwTokenResponse {
        let msg = Cw4626InstantiateMsg {
            name: "Cw4626 tokenized vault".to_string(),
            symbol: "vaultToken".to_string(),
            decimals: 6,
            marketing: None,
        };

        cw4626.instantiate(deps, Some(to_binary(&msg)?))
    }

    fn setup_and_mint(
        mint_amount: Uint128,
        recipient: Option<&Addr>,
    ) -> (OwnedDeps<MemoryStorage, MockApi, MockQuerier>, Env, Cw4626) {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let cw4626 = Cw4626 {
            address: Addr::unchecked("cw4626"),
        };

        instantiate_cw4626(cw4626.clone(), deps.as_mut()).unwrap();

        cw4626
            .mint(
                deps.as_mut(),
                &env,
                recipient.unwrap_or(&env.contract.address),
                mint_amount,
            )
            .unwrap();

        (deps, env, cw4626)
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let cw4626 = Cw4626 {
            address: Addr::unchecked("cw4626"),
        };

        instantiate_cw4626(cw4626, deps.as_mut()).unwrap();

        // Assert correct token info
        let token_info = TOKEN_INFO.load(deps.as_ref().storage).unwrap();
        assert_eq!(token_info.name, "Cw4626 tokenized vault");
        assert_eq!(token_info.symbol, "vaultToken");
        assert_eq!(token_info.decimals, 6);
        assert_eq!(token_info.total_supply, Uint128::zero());
        assert_eq!(token_info.mint, None);
    }

    #[test]
    fn test_mint_and_burn() {
        // Setup and mint 1000 tokens
        let mint_amount = Uint128::from(1000u128);
        let (mut deps, env, cw4626) = setup_and_mint(mint_amount, None);

        // Assert correct balance was minted
        let balance = BALANCES.load(&deps.storage, &env.contract.address).unwrap();
        assert_eq!(balance, mint_amount);

        // Assert correct total supply
        let token_info = TOKEN_INFO.load(&deps.storage).unwrap();
        assert_eq!(token_info.total_supply, mint_amount);

        // Try burning more than was minted
        let burn_amount = Uint128::from(5000u128);
        let res = cw4626.burn(deps.as_mut(), &env, burn_amount).unwrap_err();

        // Assert error message
        assert_eq!(
            res,
            CwTokenError::Std(StdError::Overflow {
                source: OverflowError {
                    operation: OverflowOperation::Sub,
                    operand1: mint_amount.to_string(),
                    operand2: burn_amount.to_string(),
                }
            })
        );

        // Burn correct amount
        let burn_amount = Uint128::from(500u128);
        cw4626.burn(deps.as_mut(), &env, burn_amount).unwrap();

        // Assert correct balance was burned
        let balance = BALANCES.load(&deps.storage, &env.contract.address).unwrap();
        assert_eq!(balance, mint_amount - burn_amount);

        // Assert correct total supply
        let token_info = TOKEN_INFO.load(&deps.storage).unwrap();
        assert_eq!(token_info.total_supply, mint_amount - burn_amount);
    }

    #[test]
    fn test_vault_token_queries() {
        // Setup and mint 1000 tokens
        let mint_amount = Uint128::from(1000u128);
        let (deps, env, cw4626) = setup_and_mint(mint_amount, None);

        // Assert that total supply query is correct
        let total_supply = cw4626.query_total_supply(deps.as_ref()).unwrap();
        assert_eq!(total_supply, mint_amount);

        // Assert that balance query is correct
        let balance = cw4626
            .query_balance(deps.as_ref(), &env.contract.address)
            .unwrap();
        assert_eq!(balance, mint_amount);
    }

    #[test]
    fn test_receive() {
        let sender = Addr::unchecked(SENDER);
        let info = mock_info(SENDER, &[]);

        // Setup and mint 1000 tokens
        let mint_amount = Uint128::from(1000u128);
        let (mut deps, env, cw4626) = setup_and_mint(mint_amount, Some(&sender));

        // Test receiving more than was minted
        let receive_amount = Uint128::from(5000u128);
        let res = cw4626
            .receive(deps.as_mut(), &env, &info, receive_amount)
            .unwrap_err();

        // Assert overflow error message
        assert_eq!(
            res,
            StdError::Overflow {
                source: OverflowError {
                    operation: OverflowOperation::Sub,
                    operand1: mint_amount.to_string(),
                    operand2: receive_amount.to_string(),
                }
            }
        );

        // Receive 500 tokens
        let receive_amount = Uint128::from(250u128);
        cw4626
            .receive(deps.as_mut(), &env, &info, receive_amount)
            .unwrap();

        // Assert correct balance was received
        let balance = BALANCES.load(&deps.storage, &env.contract.address).unwrap();
        assert_eq!(balance, receive_amount);

        // Assert correct balance was deducted from sender
        let balance = BALANCES.load(&deps.storage, &sender).unwrap();
        assert_eq!(balance, mint_amount - receive_amount);

        // Assert correct total supply
        let token_info = TOKEN_INFO.load(&deps.storage).unwrap();
        assert_eq!(token_info.total_supply, mint_amount);
    }

    #[test]
    fn test_to_string() {
        let cw4626 = Cw4626 {
            address: Addr::unchecked("cw4626"),
        };

        assert_eq!(cw4626.to_string(), "cw4626");
    }
}
