use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{attr, to_binary, Addr, Api, Coin, CosmosMsg, Event, Uint128};
use cw_dex::osmosis::OsmosisPool;
use cw_it::app::App as RpcRunner;
use cw_it::Cli;
use cw_vault_token::osmosis::OsmosisDenom;
use cw_vault_token::{Burn, Instantiate, Mint, VaultToken};
use osmosis_testing::cosmrs::proto::cosmwasm::wasm::v1::MsgExecuteContractResponse;
use osmosis_testing::osmosis_std::types::osmosis::tokenfactory::v1beta1::{
    MsgBurnResponse, MsgCreateDenomResponse, MsgMintResponse,
};
use osmosis_testing::{Account, Gamm, Module, OsmosisTestApp, Runner, SigningAccount, Wasm};

const TEST_CONFIG_PATH: &str = "tests/configs/osmosis.yaml";
const SUBDENOM: &str = "subdenom";

#[test]
/// Runs all tests against LocalOsmosis
pub fn test_with_localosmosis() {
    let docker: Cli = Cli::default();
    let app = RpcRunner::new(TEST_CONFIG_PATH, &docker);

    let accs = app
        .test_config
        .import_all_accounts()
        .into_values()
        .collect::<Vec<_>>();

    test_instantiate(&app, &accs);
    test_mint(&app, &accs);
    test_burn(&app, &accs);
    query_vault_supply(&app, &accs);
    query_balance(&app, &accs);
}

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

    test_instantiate(&app, &accs);
    test_mint(&app, &accs);
    test_burn(&app, &accs);
    query_vault_supply(&app, &accs);
    query_balance(&app, &accs);
}

pub fn test_instantiate<R>(app: &R, accs: &Vec<SigningAccount>)
where
    R: for<'a> Runner<'a>,
{
    let signer = &accs[0];

    let mut deps = mock_dependencies();
    let denom = OsmosisDenom::new(signer.address(), SUBDENOM.to_string());
    let sub_messages = denom.instantiate(deps.as_mut(), None).unwrap().messages;

    let cosmos_msgs: Vec<CosmosMsg> = sub_messages.into_iter().map(|x| x.msg).collect();

    let res = app
        .execute_cosmos_msgs::<MsgCreateDenomResponse>(&cosmos_msgs, signer)
        .unwrap();

    assert_eq!(res.data.new_token_denom, denom.to_string());
}

pub fn test_mint<R>(app: &R, accs: &Vec<SigningAccount>)
where
    R: for<'a> Runner<'a>,
{
    let creator = &accs[0];

    let mut deps = mock_dependencies();
    let denom = OsmosisDenom::new(creator.address(), SUBDENOM.to_string());

    let recipient = &accs[1];

    let mut env = mock_env();
    // The sender in the function is `env.contract.address`, therefore I explicitly
    // changed it. Otherwise, the message will fail upon address verification
    env.contract.address = deps.as_mut().api.addr_validate(&creator.address()).unwrap();

    let amount_to_mint = Uint128::new(10000000);

    let recipient = &deps
        .as_mut()
        .api
        .addr_validate(&recipient.address())
        .unwrap();
    let sub_messages = denom
        .mint(deps.as_mut(), &env, recipient, amount_to_mint)
        .unwrap()
        .messages;

    let cosmos_msgs: Vec<CosmosMsg> = sub_messages.into_iter().map(|x| x.msg).collect();

    let res = app
        .execute_cosmos_msgs::<MsgMintResponse>(&cosmos_msgs, creator)
        .unwrap();

    // Since the repsonse is empty, it is necessary to test the events
    assert_eq!(res.data, MsgMintResponse {});

    let mint_event = res
        .events
        .clone()
        .into_iter()
        .filter(|r| r.ty == "tf_mint")
        .collect::<Vec<Event>>();

    let mut expected_event = Event::new("tf_mint".to_string());

    expected_event = expected_event.add_attributes(vec![
        attr("mint_to_address", creator.address().to_string()),
        attr("amount", format!("{}{}", amount_to_mint, denom.to_string())),
    ]);

    // Check that the mint token event is emitted
    assert_eq!(mint_event.clone().len(), 1);
    assert_eq!(mint_event[0], expected_event);

    let transfer_events = res
        .events
        .into_iter()
        .filter(|r| r.ty == "transfer")
        .collect::<Vec<Event>>();

    expected_event = Event::new("transfer".to_string());

    expected_event = expected_event.add_attributes(vec![
        attr("recipient", recipient.to_string()),
        attr("sender", creator.address().to_string()),
        attr("amount", format!("{}{}", amount_to_mint, denom.to_string())),
    ]);

    // The last transfer event performs the transfer from creator to recipient
    assert_eq!(transfer_events.last().unwrap(), &expected_event);
}

pub fn test_burn<R>(app: &R, accs: &Vec<SigningAccount>)
where
    R: for<'a> Runner<'a>,
{
    let creator = &accs[0];

    let mut deps = mock_dependencies();
    let denom = OsmosisDenom::new(creator.address(), SUBDENOM.to_string());

    let recipient = &accs[0];

    let mut env = mock_env();
    // The sender in the function is `env.contract.address`, therefore I explicitly
    // changed it. Otherwise, the message will fail upon address verification
    env.contract.address = deps.as_mut().api.addr_validate(&creator.address()).unwrap();
    let recipient = &deps
        .as_mut()
        .api
        .addr_validate(&recipient.address())
        .unwrap();

    let amount_to_mint = Uint128::new(10000000);

    let cosmos_msgs: Vec<CosmosMsg> = denom
        .mint(deps.as_mut(), &env, recipient, amount_to_mint)
        .unwrap()
        .messages
        .into_iter()
        .map(|x| x.msg)
        .collect();

    let res = app
        .execute_cosmos_msgs::<MsgMintResponse>(&cosmos_msgs, creator)
        .unwrap();

    assert_eq!(res.data, MsgMintResponse {});

    let amount_to_burn = Uint128::new(1000000);

    let cosmos_msgs: Vec<CosmosMsg> = denom
        .burn(deps.as_mut(), &env, amount_to_burn)
        .unwrap()
        .messages
        .into_iter()
        .map(|x| x.msg)
        .collect();

    let res = app
        .execute_cosmos_msgs::<MsgBurnResponse>(&cosmos_msgs, creator)
        .unwrap();

    let burn_event = res
        .events
        .clone()
        .into_iter()
        .filter(|r| r.ty == "tf_burn")
        .collect::<Vec<Event>>();

    let mut expected_event = Event::new("tf_burn".to_string());

    expected_event = expected_event.add_attributes(vec![
        attr("burn_from_address", creator.address().to_string()),
        attr("amount", format!("{}{}", amount_to_burn, denom.to_string())),
    ]);

    // Check that the burn token event is emitted
    assert_eq!(burn_event.clone().len(), 1);
    assert_eq!(burn_event[0], expected_event);
}

pub fn query_vault_supply<R>(_app: &R, accs: &Vec<SigningAccount>)
where
    R: for<'a> Runner<'a>,
{
    let creator = &accs[0];
    let denom = OsmosisDenom::new(creator.address(), SUBDENOM.to_string());
    let deps = mock_dependencies();

    let supply = denom.query_total_supply(deps.as_ref()).unwrap();

    // Minted 10000000 twice, burned 1000000 once = (10000000 * 2) - 1000000 =
    // 19000000
    assert_eq!(supply, Uint128::new(19000000));
}

pub fn query_balance<R>(_app: &R, accs: &Vec<SigningAccount>)
where
    R: for<'a> Runner<'a>,
{
    let creator = &accs[1];
    let denom = OsmosisDenom::new(creator.address(), SUBDENOM.to_string());
    let deps = mock_dependencies();

    let balance = denom
        .query_balance(deps.as_ref(), creator.address())
        .unwrap();

    // Minted 10000000 twice, burned 1000000 once = (10000000 * 2) - 1000000 =
    // 19000000
    assert_eq!(balance, Uint128::new(19000000));
}
