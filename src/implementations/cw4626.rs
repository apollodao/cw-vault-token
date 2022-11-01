use std::fmt::Display;

use ::cw20::MarketingInfoResponse;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    attr, from_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128,
};
use cw20_base::{
    contract::query_balance,
    msg::InstantiateMsg,
    state::{TokenInfo, BALANCES, MARKETING_INFO, TOKEN_INFO},
    ContractError,
};
use cw_asset::AssetInfo;

use crate::{Burn, CwTokenResponse, CwTokenResult, Instantiate, Mint, Receive, Token};

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

        let res =
            Response::new().add_attributes(vec![attr("action", "burn"), attr("amount", amount)]);
        Ok(res)
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

impl Receive for Cw4626 {
    fn receive_vault_token(
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
