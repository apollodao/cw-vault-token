use std::fmt::Display;

use ::cw20::MarketingInfoResponse;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    from_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
};
use cw20_base::{
    allowances::execute_burn_from,
    contract::{execute_transfer, query_balance},
    msg::InstantiateMsg,
    state::{TokenInfo, BALANCES, MARKETING_INFO, TOKEN_INFO},
    ContractError,
};
use cw_asset::AssetInfo;

use crate::{
    AssertReceived, Burn, CwTokenError, CwTokenResponse, CwTokenResult, Instantiate, Mint, Token,
};

#[cw_serde]
pub struct Cw4626(pub Addr);

impl Display for Cw4626 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Cw4626> for AssetInfo {
    fn from(cw20_asset: Cw4626) -> Self {
        AssetInfo::Cw20(cw20_asset.0)
    }
}

impl TryFrom<AssetInfo> for Cw4626 {
    type Error = StdError;

    fn try_from(asset_info: AssetInfo) -> StdResult<Self> {
        match asset_info {
            AssetInfo::Cw20(address) => Ok(Cw4626(address)),
            _ => Err(StdError::generic_err(
                "Cannot convert non-cw20 asset to Cw20.",
            )),
        }
    }
}

impl Token for Cw4626 {
    fn transfer<A: Into<String>>(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: A,
        amount: Uint128,
    ) -> Result<Response, CwTokenError> {
        Ok(execute_transfer(deps, env, info, recipient.into(), amount)?)
    }

    fn query_balance<A: Into<String>>(&self, deps: Deps, address: A) -> CwTokenResult<Uint128> {
        Ok(query_balance(deps, address.into())?.balance)
    }

    fn query_total_supply(&self, deps: Deps) -> CwTokenResult<Uint128> {
        Ok(TOKEN_INFO.load(deps.storage)?.total_supply)
    }

    fn is_native() -> bool {
        false
    }
}

/// Mints the specified amount of tokens for the recipient.
/// The contract should validate that the recipient is allowed to do this before
/// calling the function, i.e. make sure that the recipient has sent sufficient
/// assets to the vault, or perform a transfer_from, or similar.
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
        let rcpt_addr = deps.api.addr_validate(&recipient.clone())?;
        BALANCES.update(
            deps.storage,
            &rcpt_addr,
            |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
        )?;

        let res = Response::new()
            .add_attribute("action", "mint")
            .add_attribute("to", recipient)
            .add_attribute("amount", amount);
        Ok(res)
    }
}

impl Burn for Cw4626 {
    fn burn(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        owner: &Addr,
        amount: Uint128,
    ) -> CwTokenResponse {
        Ok(execute_burn_from(
            deps,
            env.clone(),
            info.clone(),
            owner.to_string(),
            amount,
        )?)
    }
}

impl Instantiate for Cw4626 {
    fn instantiate(&self, deps: DepsMut, init_info: Option<Binary>) -> CwTokenResponse {
        let msg: InstantiateMsg =
            from_binary(&init_info.ok_or(StdError::generic_err("init_info requried for Cw4626"))?)?;

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
                logo: None, //For some reason all the logo validation functions are private. We ignore logo info for now.
            };
            MARKETING_INFO.save(deps.storage, &data)?;
        }

        Ok(Response::default())
    }
}

impl AssertReceived for Cw4626 {
    fn assert_received(&self, deps: Deps, info: &MessageInfo, amount: Uint128) -> StdResult<()> {
        let balance = BALANCES
            .may_load(deps.storage, &info.sender)?
            .unwrap_or_default();

        if balance != amount {
            return Err(StdError::generic_err(format!(
                "Tried to use {} tokens, but only {} tokens are available",
                amount, balance
            )));
        }
        Ok(())
    }
}
