use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier};
use cosmwasm_std::{to_binary, Coin, DepsMut, Empty, MemoryStorage, OwnedDeps, Uint128};
use cw20_base::state::{TokenInfo, TOKEN_INFO};

use cw_vault_token::cw4626::{Cw4626, Cw4626InstantiateMsg};

use cw_vault_token::{Burn, Instantiate, Mint, VaultToken};

use cw_it::osmosis_test_tube::OsmosisTestApp;
use cw_it::test_tube::{Account, Runner, SigningAccount};

const SUBDENOM: &str = "subdenom";

#[test]
/// Runs all tests against the Osmosis bindings
pub fn test_with_osmosis_bindings() {
    let app = OsmosisTestApp::default();

    let accs = app
        .init_accounts(
            &[
                Coin::new(1_000_000_000_000, "uatom"),
                Coin::new(1_000_000_000_000, "uosmo"),
            ],
            2,
        )
        .unwrap();

    let (cw4626, mut deps) = test_instantiate(&app, &accs);
    test_mint(&app, &accs, &cw4626, deps.as_mut());
    test_burn(&app, &accs, &cw4626, deps.as_mut());
    query_vault_supply(&app, &accs, &cw4626, deps.as_mut());
    query_balance(&app, &accs, &cw4626, deps.as_mut());
}

pub fn test_instantiate<R>(
    _app: &R,
    _accs: &[SigningAccount],
) -> (
    Cw4626,
    OwnedDeps<MemoryStorage, MockApi, MockQuerier<Empty>, Empty>,
)
where
    R: for<'a> Runner<'a>,
{
    let mut deps = mock_dependencies();
    let cw4626 = Cw4626::new(&mock_env());
    let init_info = to_binary(&Cw4626InstantiateMsg {
        name: SUBDENOM.to_string(),
        symbol: "VAULT".to_string(),
        decimals: 6,
        marketing: None,
    })
    .unwrap();

    cw4626.instantiate(deps.as_mut(), Some(init_info)).unwrap();

    let token_info = TOKEN_INFO.load(&deps.storage).unwrap();

    assert_eq!(
        token_info,
        TokenInfo {
            name: SUBDENOM.to_string(),
            symbol: "VAULT".to_string(),
            decimals: 6,
            total_supply: Uint128::zero(),
            mint: None
        }
    );

    (cw4626, deps)
}

pub fn test_mint<R>(_app: &R, accs: &[SigningAccount], cw4626: &Cw4626, mut deps: DepsMut)
where
    R: for<'a> Runner<'a>,
{
    let creator = &accs[0];

    let recipient = &accs[1];

    let mut env = mock_env();
    // The sender in the function is `env.contract.address`, therefore I explicitly
    // changed it. Otherwise, the message will fail upon address verification.
    env.contract.address = deps.api.addr_validate(&creator.address()).unwrap();

    let amount_to_mint = Uint128::new(10000000);

    let recipient = deps.api.addr_validate(&recipient.address()).unwrap();
    cw4626
        .mint(deps.branch(), &env, &recipient, amount_to_mint)
        .unwrap();

    let supply = cw4626.query_total_supply(deps.as_ref()).unwrap();
    assert_eq!(supply, Uint128::new(10000000));

    let balance = cw4626.query_balance(deps.as_ref(), recipient).unwrap();
    assert_eq!(balance, Uint128::new(10000000));
}

pub fn test_burn<R>(_app: &R, accs: &[SigningAccount], cw4626: &Cw4626, mut deps: DepsMut)
where
    R: for<'a> Runner<'a>,
{
    let creator = &accs[1];

    let recipient = &accs[1];

    let mut env = mock_env();
    // The sender in the function is `env.contract.address`, therefore I explicitly
    // changed it. Otherwise, the message will fail upon address verification
    env.contract.address = deps.api.addr_validate(&creator.address()).unwrap();

    let amount_to_burn = Uint128::new(1000000);

    cw4626.burn(deps.branch(), &env, amount_to_burn).unwrap();

    let supply = cw4626.query_total_supply(deps.as_ref()).unwrap();
    assert_eq!(supply, Uint128::new(9000000));

    let balance = cw4626
        .query_balance(deps.as_ref(), recipient.address())
        .unwrap();
    assert_eq!(balance, Uint128::new(9000000));
}

pub fn query_vault_supply<R>(_app: &R, _accs: &[SigningAccount], cw4626: &Cw4626, deps: DepsMut)
where
    R: for<'a> Runner<'a>,
{
    let supply = cw4626.query_total_supply(deps.as_ref()).unwrap();
    // Minted 10000000 twice, burned 1000000 once = (10000000 * 2) - 1000000 =
    // 19000000
    assert_eq!(supply, Uint128::new(9000000));
}

pub fn query_balance<R>(_app: &R, accs: &[SigningAccount], cw4626: &Cw4626, deps: DepsMut)
where
    R: for<'a> Runner<'a>,
{
    let recipient = &accs[1];

    let balance = cw4626
        .query_balance(deps.as_ref(), recipient.address())
        .unwrap();

    // Minted 10000000 twice, burned 1000000 once = (10000000 * 2) - 1000000 =
    // 19000000
    assert_eq!(balance, Uint128::new(9000000));
}
