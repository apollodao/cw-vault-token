use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{attr, Api, Attribute, Coin, CosmosMsg, Deps, Env, Event, Response, Uint128};

use cw_vault_token::osmosis::OsmosisDenom;
use cw_vault_token::{Burn, Instantiate, Mint, VaultToken};

use cw_it::osmosis_std::types::osmosis::tokenfactory::v1beta1::{
    MsgBurnResponse, MsgCreateDenomResponse, MsgMintResponse,
};
use cw_it::osmosis_test_tube::OsmosisTestApp;
use cw_it::test_tube::{Account, Runner, SigningAccount};

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
struct TokenRobot<'a, R: Runner<'a>, T: VaultToken + Clone> {
    app: &'a R,
    denom: &'a T,
    last_events: Vec<Event>,
}

impl<'a, R: Runner<'a>> TokenRobot<'a, R, OsmosisDenom> {
    pub fn new(app: &'a R, denom: &'a OsmosisDenom) -> Self {
        Self {
            app,
            denom,
            last_events: vec![],
        }
    }

    fn instantiate<S: ::prost::Message + Default>(self, signer: &SigningAccount) -> Self {
        let response = self
            .denom
            .instantiate(mock_dependencies().as_mut(), None)
            .unwrap();

        self.execute_response::<S>(signer, response)
    }

    fn mint<S: ::prost::Message + Default>(
        self,
        signer: &SigningAccount,
        recipient: &str,
        amount: Uint128,
    ) -> Self {
        let mut deps = mock_dependencies();
        let env = mock_env_with_address(deps.as_ref(), &signer.address());

        let recipient = deps.api.addr_validate(recipient).unwrap();
        let response = self
            .denom
            .clone()
            .mint(deps.as_mut(), &env, &recipient, amount)
            .unwrap();
        self.execute_response::<S>(signer, response)
    }

    fn burn<S: ::prost::Message + Default>(self, signer: &SigningAccount, amount: Uint128) -> Self {
        let mut deps = mock_dependencies();
        let env = mock_env_with_address(deps.as_ref(), &signer.address());

        let response = self.denom.burn(deps.as_mut(), &env, amount).unwrap();

        self.execute_response::<S>(signer, response)
    }

    fn execute_response<S: ::prost::Message + Default>(
        mut self,
        signer: &SigningAccount,
        res: Response,
    ) -> Self {
        let cosmos_msgs: Vec<CosmosMsg> = res.messages.into_iter().map(|x| x.msg).collect();

        let execute_res = self
            .app
            .execute_cosmos_msgs::<S>(&cosmos_msgs, signer)
            .unwrap();

        self.last_events = execute_res.events;

        self
    }

    fn assert_event(self, expected_type: &str, expected_attributes: Vec<Attribute>) -> Self {
        let expected_event = &Event::new(expected_type).add_attributes(expected_attributes);
        match self.last_events.contains(expected_event) {
            true => self,
            false => panic!("Event not found. Expected {:?}", expected_event),
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

    TokenRobot::new(&app, &denom)
        .instantiate::<MsgCreateDenomResponse>(creator)
        .mint::<MsgMintResponse>(signer, &recipient.address(), amount)
        .assert_event(
            "tf_mint",
            vec![
                attr("mint_to_address", creator.address()),
                attr("amount", format!("{}{}", amount, denom)),
            ],
        )
        .assert_event(
            "transfer",
            vec![
                attr("recipient", recipient.address()),
                attr("sender", creator.address()),
                attr("amount", format!("{}{}", amount, denom)),
            ],
        );
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

    TokenRobot::new(&app, &denom)
        .instantiate::<MsgCreateDenomResponse>(creator)
        .mint::<MsgMintResponse>(creator, &recipient.address(), Uint128::from(1000000u128))
        .burn::<MsgBurnResponse>(signer, amount)
        .assert_event(
            "tf_burn",
            vec![
                attr("burn_from_address", creator.address()),
                attr("amount", format!("{}{}", amount, denom)),
            ],
        );
}

// Tests disabled because Querier no longer implemented for OsmosisTestApp
// #[test_case(0 ; "total supply")]
// #[test_case(1 ; "balance")]
// fn query(query: usize) {
//     let (app, accs) = setup();
//     let creator = &accs[0];
//     let recipient = &accs[1];
//     let amount = Uint128::from(1000000u128);
//     let denom = OsmosisDenom::new(creator.address(), SUBDENOM.to_string());

//     let robot = TokenRobot::new(&app, &denom)
//         .instantiate::<MsgCreateDenomResponse>(creator)
//         .mint::<MsgMintResponse>(creator, &recipient.address(), amount);

//     let query_result = match query {
//         0 => robot.query_total_supply(),
//         1 => robot.query_balance(&recipient.address()),
//         _ => panic!("invalid query"),
//     };

//     assert_eq!(query_result, amount);
// }
