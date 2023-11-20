#![cfg(not(tarpaulin_include))]
mod factory_helper;
use cosmwasm_std::Addr;
use cw_multi_test::{App, ContractWrapper, Executor};
use ura::contracts::factory::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, PairConfig, PairType, QueryMsg,
};

use crate::factory_helper::FactoryHelper;

fn mock_app() -> App {
    App::default()
}

fn store_factory_code(app: &mut App) -> u64 {
    let factory_contract = Box::new(
        ContractWrapper::new_with_empty(
            ura_factory::executes::execute,
            ura_factory::contract::instantiate,
            ura_factory::queries::query,
        )
        .with_reply_empty(ura_factory::contract::reply),
    );

    app.store_code(factory_contract)
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();

    let owner = Addr::unchecked("owner");

    let factory_code_id = store_factory_code(&mut app);

    let pair_configs = vec![PairConfig {
        code_id: 321,
        pair_type: PairType::Xyk,
        total_fee_bps: 100,
        is_disabled: false,
        is_controller_disabled: false,
    }];

    let msg = InstantiateMsg {
        pair_configs: pair_configs.clone(),
        owner: owner.to_string(),
        controller_address: Some(String::from("controller")),
        coin_registry_address: "coin_registry".to_string(),
        fee_address: None,
        token_code_id: 123u64,
    };

    let factory_instance = app
        .instantiate_contract(
            factory_code_id,
            Addr::unchecked(owner.clone()),
            &msg,
            &[],
            "factory",
            None,
        )
        .unwrap();

    let msg = QueryMsg::Config {};
    let config_res: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&factory_instance, &msg)
        .unwrap();

    assert_eq!(pair_configs, config_res.pair_configs);
    assert_eq!(owner, config_res.owner);
}

#[test]
fn update_config() {
    let mut app = mock_app();
    let owner = Addr::unchecked("owner");
    let mut helper = FactoryHelper::init(&mut app, &owner);

    // Update config
    helper
        .update_config(
            &mut app,
            &owner,
            Some(200u64),
            Some("fee".to_string()),
            Some("controller".to_string()),
            None,
            None,
        )
        .unwrap();

    let config_res: ConfigResponse = app
        .wrap()
        .query_wasm_smart(&helper.factory, &QueryMsg::Config {})
        .unwrap();

    assert_eq!(
        "controller",
        config_res.controller_address.unwrap().to_string()
    );

    // Unauthorized err
    let res = helper
        .update_config(
            &mut app,
            &Addr::unchecked("not_owner"),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap_err();
    assert_eq!(res.root_cause().to_string(), "Unauthorized");
}

#[test]
fn check_update_owner() {
    let mut app = mock_app();
    let owner = Addr::unchecked("owner");
    let helper = FactoryHelper::init(&mut app, &owner);

    let new_owner = String::from("new_owner");

    // New owner
    let msg = ExecuteMsg::ProposeNewOwner {
        owner: new_owner.clone(),
        expires_in: 100, // seconds
    };

    // Unauthed check
    let err = app
        .execute_contract(
            Addr::unchecked("not_owner"),
            helper.factory.clone(),
            &msg,
            &[],
        )
        .unwrap_err();
    assert_eq!(err.root_cause().to_string(), "Generic error: Unauthorized");

    // Claim before proposal
    let err = app
        .execute_contract(
            Addr::unchecked(new_owner.clone()),
            helper.factory.clone(),
            &ExecuteMsg::ClaimOwnership {},
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Generic error: Ownership proposal not found"
    );

    // Propose new owner
    app.execute_contract(Addr::unchecked("owner"), helper.factory.clone(), &msg, &[])
        .unwrap();

    // Claim from invalid addr
    let err = app
        .execute_contract(
            Addr::unchecked("invalid_addr"),
            helper.factory.clone(),
            &ExecuteMsg::ClaimOwnership {},
            &[],
        )
        .unwrap_err();
    assert_eq!(err.root_cause().to_string(), "Generic error: Unauthorized");

    // Drop ownership proposal
    let err = app
        .execute_contract(
            Addr::unchecked(new_owner.clone()),
            helper.factory.clone(),
            &ExecuteMsg::DropOwnershipProposal {},
            &[],
        )
        .unwrap_err();
    // new_owner is not an owner yet
    assert_eq!(err.root_cause().to_string(), "Generic error: Unauthorized");

    app.execute_contract(
        owner.clone(),
        helper.factory.clone(),
        &ExecuteMsg::DropOwnershipProposal {},
        &[],
    )
    .unwrap();

    // Try to claim ownership
    let err = app
        .execute_contract(
            Addr::unchecked(new_owner.clone()),
            helper.factory.clone(),
            &ExecuteMsg::ClaimOwnership {},
            &[],
        )
        .unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        "Generic error: Ownership proposal not found"
    );

    // Propose new owner again
    app.execute_contract(Addr::unchecked("owner"), helper.factory.clone(), &msg, &[])
        .unwrap();
    // Claim ownership
    app.execute_contract(
        Addr::unchecked(new_owner.clone()),
        helper.factory.clone(),
        &ExecuteMsg::ClaimOwnership {},
        &[],
    )
    .unwrap();

    // Let's query the contract state
    let msg = QueryMsg::Config {};
    let res: ConfigResponse = app.wrap().query_wasm_smart(&helper.factory, &msg).unwrap();

    assert_eq!(res.owner, new_owner)
}
