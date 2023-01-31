

use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockStorage};
use cosmwasm_std::{
    attr, Api, Coin, CosmosMsg, Deps, Empty, Env,
    Event, Querier, QuerierWrapper,
    Uint128,
};



use cw_vault_token::osmosis::OsmosisDenom;
use cw_vault_token::{Burn, Instantiate, Mint, VaultToken};



use osmosis_testing::osmosis_std::types::osmosis::tokenfactory::v1beta1::{
    MsgBurnResponse, MsgCreateDenomResponse, MsgMintResponse,
};
use osmosis_testing::{Account, ExecuteResponse, Module, OsmosisTestApp, Runner, SigningAccount};

use test_case::test_case;

const SUBDENOM: &str = "subdenom";

/// Runs all tests against the Osmosis bindings
pub fn setup() -> (OsmosisTestApp, Vec<SigningAccount>) {
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

    (app, accs)
}

fn mock_env_with_address(deps: Deps, address: &str) -> Env {
    let mut env = mock_env();
    env.contract.address = deps.api.addr_validate(address).unwrap();
    env
}

#[derive(Clone, Debug)]
struct TokenRobot<'a, R: Runner<'a> + Querier, T: VaultToken + Clone> {
    app: &'a R,
    denom: &'a T,
}

impl<'a, R: Runner<'a> + Querier, T: VaultToken + Clone> TokenRobot<'a, R, T> {
    pub fn new(app: &'a R, denom: &'a T) -> Self {
        Self { app, denom }
    }

    fn instantiate<S: ::prost::Message + Default>(
        self,
        signer: &SigningAccount,
    ) -> (Self, ExecuteResponse<S>) {
        let sub_msgs = self
            .denom
            .instantiate(mock_dependencies().as_mut(), None)
            .unwrap()
            .messages;

        let cosmos_msgs: Vec<CosmosMsg> = sub_msgs.into_iter().map(|x| x.msg).collect();

        let res = self
            .app
            .execute_cosmos_msgs::<S>(&cosmos_msgs, signer)
            .unwrap();

        (self, res)
    }

    fn mint<S: ::prost::Message + Default>(
        self,
        signer: &SigningAccount,
        recipient: &str,
        amount: Uint128,
    ) -> (Self, ExecuteResponse<S>) {
        let mut deps = mock_dependencies();
        let env = mock_env_with_address(deps.as_ref(), &signer.address());

        let recipient = deps.api.addr_validate(recipient).unwrap();
        let sub_messages = self
            .denom
            .mint(deps.as_mut(), &env, &recipient, amount)
            .unwrap()
            .messages;

        let cosmos_msgs: Vec<CosmosMsg> = sub_messages.into_iter().map(|x| x.msg).collect();

        let res = self
            .app
            .execute_cosmos_msgs::<S>(&cosmos_msgs, signer)
            .unwrap();

        (self, res)
    }

    fn burn<S: ::prost::Message + Default>(
        self,
        signer: &SigningAccount,
        amount: Uint128,
    ) -> (Self, ExecuteResponse<S>) {
        let mut deps = mock_dependencies();
        let env = mock_env_with_address(deps.as_ref(), &signer.address());

        let sub_messages = self
            .denom
            .burn(deps.as_mut(), &env, amount)
            .unwrap()
            .messages;

        let cosmos_msgs: Vec<CosmosMsg> = sub_messages.into_iter().map(|x| x.msg).collect();

        let res = self
            .app
            .execute_cosmos_msgs::<S>(&cosmos_msgs, signer)
            .unwrap();

        (self, res)
    }

    fn query_balance(&self, address: &str) -> Uint128 {
        let deps = RunnerMockDeps::new(self.app);

        self.denom.query_balance(deps.as_ref(), address).unwrap()
    }

    fn query_total_supply(&self) -> Uint128 {
        let deps = RunnerMockDeps::new(self.app);

        self.denom.query_total_supply(deps.as_ref()).unwrap()
    }
}

struct RunnerMockDeps<'a, Q: Querier> {
    pub storage: MockStorage,
    pub api: MockApi,
    pub querier: &'a Q,
}

impl<'a, Q: Querier> RunnerMockDeps<'a, Q> {
    pub fn new(querier: &'a Q) -> Self {
        Self {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier,
        }
    }
    pub fn as_ref(&'_ self) -> Deps<'_, Empty> {
        Deps {
            storage: &self.storage,
            api: &self.api,
            querier: QuerierWrapper::new(self.querier),
        }
    }
}

#[test_case(0 ; "signer is owner")]
#[test_case(1 => panics ; "signer is not owner")]
pub fn instantiate(owner_idx: usize) {
    let (app, accs) = setup();
    let signer = &accs[0];
    let owner = &accs[owner_idx];
    let denom = OsmosisDenom::new(owner.address(), SUBDENOM.to_string());

    TokenRobot::new(&app, &denom).instantiate::<MsgCreateDenomResponse>(signer);
}

#[test_case(0, Uint128::from(1000000u128) ; "executed by owner")]
#[test_case(1, Uint128::from(1000000u128) => panics ; "executed by non-owner")]
#[test_case(0, Uint128::zero() => panics ; "zero amount")]
fn mint(signer_idx: usize, amount: Uint128) {
    let (app, accs) = setup();
    let creator = &accs[0];
    let signer = &accs[signer_idx];
    let recipient = &accs[1];
    let denom = OsmosisDenom::new(creator.address(), SUBDENOM.to_string());

    let (_, mint_res) = TokenRobot::new(&app, &denom)
        .instantiate::<MsgCreateDenomResponse>(creator)
        .0
        .mint::<MsgMintResponse>(signer, &recipient.address(), amount);

    let mint_event = mint_res
        .events
        .clone()
        .into_iter()
        .filter(|r| r.ty == "tf_mint")
        .collect::<Vec<Event>>();

    let expected_event = Event::new("tf_mint".to_string()).add_attributes(vec![
        attr("mint_to_address", creator.address()),
        attr("amount", format!("{}{}", amount, denom)),
    ]);

    // Check that the mint token event is emitted
    assert_eq!(mint_event.len(), 1);
    assert_eq!(mint_event[0], expected_event);

    let transfer_events = mint_res
        .events
        .into_iter()
        .filter(|r| r.ty == "transfer")
        .collect::<Vec<Event>>();

    let expected_event = Event::new("transfer".to_string()).add_attributes(vec![
        attr("recipient", recipient.address()),
        attr("sender", creator.address()),
        attr("amount", format!("{}{}", amount, denom)),
    ]);

    // The last transfer event performs the transfer from creator to recipient
    assert_eq!(transfer_events.last().unwrap(), &expected_event);
}

#[test_case(0, 0, Uint128::from(1000000u128) ; "executed by owner")]
#[test_case(1, 1, Uint128::from(1000000u128) => panics ; "executed by non-owner")]
#[test_case(0, 0, Uint128::zero() => panics ; "zero amount")]
#[test_case(0, 0, Uint128::from(2000000u128) => panics ; "insufficient balance")]
fn burn(signer_idx: usize, recipient_idx: usize, amount: Uint128) {
    let (app, accs) = setup();
    let creator = &accs[0];
    let signer = &accs[signer_idx];
    let recipient = &accs[recipient_idx];
    let denom = OsmosisDenom::new(creator.address(), SUBDENOM.to_string());

    let (_, burn_res) = TokenRobot::new(&app, &denom)
        .instantiate::<MsgCreateDenomResponse>(creator)
        .0
        .mint::<MsgMintResponse>(creator, &recipient.address(), Uint128::from(1000000u128))
        .0
        .burn::<MsgBurnResponse>(signer, amount);

    let burn_event = burn_res
        .events
        .into_iter()
        .filter(|r| r.ty == "tf_burn")
        .collect::<Vec<Event>>();

    let expected_event = Event::new("tf_burn".to_string()).add_attributes(vec![
        attr("burn_from_address", creator.address()),
        attr("amount", format!("{}{}", amount, denom)),
    ]);

    // Check that the burn token event is emitted
    assert_eq!(burn_event.len(), 1);
    assert_eq!(burn_event[0], expected_event);
}

#[test_case(0 ; "total supply")]
#[test_case(1 ; "balance")]
fn query(query: usize) {
    let (app, accs) = setup();
    let creator = &accs[0];
    let recipient = &accs[1];
    let amount = Uint128::from(1000000u128);
    let denom = OsmosisDenom::new(creator.address(), SUBDENOM.to_string());

    let robot = TokenRobot::new(&app, &denom)
        .instantiate::<MsgCreateDenomResponse>(creator)
        .0
        .mint::<MsgMintResponse>(creator, &recipient.address(), amount)
        .0;

    let query_result = match query {
        0 => robot.query_total_supply(),
        1 => robot.query_balance(&recipient.address()),
        _ => panic!("invalid query"),
    };

    assert_eq!(query_result, amount);
}
