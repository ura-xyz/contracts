use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_json, to_json_binary, Addr, Coin, Decimal, ReplyOn, SubMsg, Uint128, WasmMsg,
};

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::testing::mock_querier::mock_dependencies;

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use ura::structs::asset_info::{native_asset_info, AssetInfo};

use ura::contracts::router::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
    SimulateSwapOperationsResponse, SwapOperation, MAX_SWAP_OPERATIONS,
};

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        ura_factory: String::from("urafactory"),
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    // We can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // It worked, let's query the state
    let config: ConfigResponse =
        from_json(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!("urafactory", config.ura_factory.as_str());
}

#[test]
fn execute_swap_operations() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        ura_factory: String::from("urafactory"),
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    // We can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    let msg = ExecuteMsg::ExecuteSwapOperations {
        operations: vec![],
        to: None,
        minimum_receive: None,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(res, ContractError::MustProvideOperations {});

    let msg = ExecuteMsg::ExecuteSwapOperations {
        operations: vec![
            SwapOperation {
                offer_asset_info: AssetInfo::NativeToken {
                    denom: "ukrw".to_string(),
                },
                ask_asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset0001"),
                },
            },
            SwapOperation {
                offer_asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset0001"),
                },
                ask_asset_info: AssetInfo::NativeToken {
                    denom: "uluna".to_string(),
                },
            },
            SwapOperation {
                offer_asset_info: AssetInfo::NativeToken {
                    denom: "uluna".to_string(),
                },
                ask_asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset0002"),
                },
            },
        ],
        to: None,
        minimum_receive: Some(Uint128::from(1000000u128)),
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: String::from(MOCK_CONTRACT_ADDR),
                    funds: vec![],
                    msg: to_json_binary(&ExecuteMsg::ExecuteSwapOperation {
                        operation: SwapOperation {
                            offer_asset_info: AssetInfo::NativeToken {
                                denom: "ukrw".to_string(),
                            },
                            ask_asset_info: AssetInfo::Token {
                                contract_addr: Addr::unchecked("asset0001"),
                            },
                        },
                        to: None,
                    })
                    .unwrap(),
                }
                .into(),
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Never,
            },
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: String::from(MOCK_CONTRACT_ADDR),
                    funds: vec![],
                    msg: to_json_binary(&ExecuteMsg::ExecuteSwapOperation {
                        operation: SwapOperation {
                            offer_asset_info: AssetInfo::Token {
                                contract_addr: Addr::unchecked("asset0001"),
                            },
                            ask_asset_info: AssetInfo::NativeToken {
                                denom: "uluna".to_string(),
                            },
                        },
                        to: None,
                    })
                    .unwrap(),
                }
                .into(),
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Never,
            },
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: String::from(MOCK_CONTRACT_ADDR),
                    funds: vec![],
                    msg: to_json_binary(&ExecuteMsg::ExecuteSwapOperation {
                        operation: SwapOperation {
                            offer_asset_info: AssetInfo::NativeToken {
                                denom: "uluna".to_string(),
                            },
                            ask_asset_info: AssetInfo::Token {
                                contract_addr: Addr::unchecked("asset0002"),
                            },
                        },
                        to: Some(String::from("addr0000")),
                    })
                    .unwrap(),
                }
                .into(),
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Never,
            },
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: String::from(MOCK_CONTRACT_ADDR),
                    funds: vec![],
                    msg: to_json_binary(&ExecuteMsg::AssertMinimumReceive {
                        asset_info: AssetInfo::Token {
                            contract_addr: Addr::unchecked("asset0002"),
                        },
                        prev_balance: Uint128::zero(),
                        minimum_receive: Uint128::from(1000000u128),
                        receiver: String::from("addr0000"),
                    })
                    .unwrap(),
                }
                .into(),
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Never,
            },
        ]
    );

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: String::from("addr0000"),
        amount: Uint128::from(1000000u128),
        msg: to_json_binary(&Cw20HookMsg::ExecuteSwapOperations {
            operations: vec![
                SwapOperation {
                    offer_asset_info: AssetInfo::NativeToken {
                        denom: "ukrw".to_string(),
                    },
                    ask_asset_info: AssetInfo::Token {
                        contract_addr: Addr::unchecked("asset0001"),
                    },
                },
                SwapOperation {
                    offer_asset_info: AssetInfo::Token {
                        contract_addr: Addr::unchecked("asset0001"),
                    },
                    ask_asset_info: AssetInfo::NativeToken {
                        denom: "uluna".to_string(),
                    },
                },
                SwapOperation {
                    offer_asset_info: AssetInfo::NativeToken {
                        denom: "uluna".to_string(),
                    },
                    ask_asset_info: AssetInfo::Token {
                        contract_addr: Addr::unchecked("asset0002"),
                    },
                },
            ],
            to: Some(String::from("addr0002")),
            minimum_receive: None,
        })
        .unwrap(),
    });

    let env = mock_env();
    let info = mock_info("asset0000", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: String::from(MOCK_CONTRACT_ADDR),
                    funds: vec![],
                    msg: to_json_binary(&ExecuteMsg::ExecuteSwapOperation {
                        operation: SwapOperation {
                            offer_asset_info: AssetInfo::NativeToken {
                                denom: "ukrw".to_string(),
                            },
                            ask_asset_info: AssetInfo::Token {
                                contract_addr: Addr::unchecked("asset0001"),
                            },
                        },
                        to: None,
                    })
                    .unwrap(),
                }
                .into(),
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Never,
            },
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: String::from(MOCK_CONTRACT_ADDR),
                    funds: vec![],
                    msg: to_json_binary(&ExecuteMsg::ExecuteSwapOperation {
                        operation: SwapOperation {
                            offer_asset_info: AssetInfo::Token {
                                contract_addr: Addr::unchecked("asset0001"),
                            },
                            ask_asset_info: AssetInfo::NativeToken {
                                denom: "uluna".to_string(),
                            },
                        },
                        to: None,
                    })
                    .unwrap(),
                }
                .into(),
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Never,
            },
            SubMsg {
                msg: WasmMsg::Execute {
                    contract_addr: String::from(MOCK_CONTRACT_ADDR),
                    funds: vec![],
                    msg: to_json_binary(&ExecuteMsg::ExecuteSwapOperation {
                        operation: SwapOperation {
                            offer_asset_info: AssetInfo::NativeToken {
                                denom: "uluna".to_string(),
                            },
                            ask_asset_info: AssetInfo::Token {
                                contract_addr: Addr::unchecked("asset0002"),
                            },
                        },
                        to: Some(String::from("addr0002")),
                    })
                    .unwrap(),
                }
                .into(),
                id: 0,
                gas_limit: None,
                reply_on: ReplyOn::Never,
            }
        ]
    );
}

#[test]
fn execute_swap_operation() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        ura_factory: String::from("urafactory"),
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    // We can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    deps.querier
        .with_ura_pairs(&[(&"uusdasset".to_string(), &String::from("pair"))]);
    deps.querier.with_balance(&[(
        &String::from(MOCK_CONTRACT_ADDR),
        &[Coin {
            amount: Uint128::new(1000000u128),
            denom: "uusd".to_string(),
        }],
    )]);

    deps.querier
        .with_ura_pairs(&[(&"assetuusd".to_string(), &String::from("pair"))]);
    deps.querier.with_token_balances(&[(
        &String::from("asset"),
        &[(
            &String::from(MOCK_CONTRACT_ADDR),
            &Uint128::new(1000000u128),
        )],
    )]);
    let msg = ExecuteMsg::ExecuteSwapOperation {
        operation: SwapOperation {
            offer_asset_info: AssetInfo::Token {
                contract_addr: Addr::unchecked("asset"),
            },
            ask_asset_info: AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
        },
        to: Some(String::from("addr0000")),
    };
    let env = mock_env();
    let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg {
            msg: WasmMsg::Execute {
                contract_addr: String::from("asset"),
                funds: vec![],
                msg: to_json_binary(&Cw20ExecuteMsg::Send {
                    contract: String::from("pair"),
                    amount: Uint128::new(1000000u128),
                    msg: to_json_binary(&ura::contracts::pair::Cw20HookMsg::Swap {
                        ask_asset_info: Some(native_asset_info("uusd".to_string())),
                        belief_price: None,
                        max_spread: Some(Decimal::one()),
                        to: Some(String::from("addr0000")),
                    })
                    .unwrap()
                })
                .unwrap()
            }
            .into(),
            id: 0,
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }]
    );
}

#[test]
fn query_buy_with_routes() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        ura_factory: String::from("urafactory"),
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    // We can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    let msg = QueryMsg::SimulateSwapOperations {
        offer_amount: Uint128::from(1000000u128),
        operations: vec![
            SwapOperation {
                offer_asset_info: AssetInfo::NativeToken {
                    denom: "ukrw".to_string(),
                },
                ask_asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset0000"),
                },
            },
            SwapOperation {
                offer_asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset0000"),
                },
                ask_asset_info: AssetInfo::NativeToken {
                    denom: "uluna".to_string(),
                },
            },
        ],
    };
    deps.querier.with_ura_pairs(&[
        (&"ukrwasset0000".to_string(), &String::from("pair0000")),
        (&"asset0000uluna".to_string(), &String::from("pair0001")),
    ]);

    let res: SimulateSwapOperationsResponse =
        from_json(&query(deps.as_ref(), env.clone(), msg).unwrap()).unwrap();
    assert_eq!(
        res,
        SimulateSwapOperationsResponse {
            amount: Uint128::from(1000000u128)
        }
    );

    assert_eq!(
        res,
        SimulateSwapOperationsResponse {
            amount: Uint128::from(1000000u128),
        }
    );
}

#[test]
fn assert_minimum_receive_native_token() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.with_balance(&[(
        &String::from("addr0000"),
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1000000u128),
        }],
    )]);

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    // Success
    let msg = ExecuteMsg::AssertMinimumReceive {
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        prev_balance: Uint128::zero(),
        minimum_receive: Uint128::from(1000000u128),
        receiver: String::from("addr0000"),
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Assertion failed; native token
    let msg = ExecuteMsg::AssertMinimumReceive {
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
        prev_balance: Uint128::zero(),
        minimum_receive: Uint128::from(1000001u128),
        receiver: String::from("addr0000"),
    };
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(
        res,
        ContractError::AssertionMinimumReceive {
            receive: Uint128::new(1000001),
            amount: Uint128::new(1000000),
        }
    );
}

#[test]
fn assert_minimum_receive_token() {
    let mut deps = mock_dependencies(&[]);

    deps.querier.with_token_balances(&[(
        &String::from("token0000"),
        &[(&String::from("addr0000"), &Uint128::from(1000000u128))],
    )]);

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    // Success
    let msg = ExecuteMsg::AssertMinimumReceive {
        asset_info: AssetInfo::Token {
            contract_addr: Addr::unchecked("token0000"),
        },
        prev_balance: Uint128::zero(),
        minimum_receive: Uint128::from(1000000u128),
        receiver: String::from("addr0000"),
    };
    let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    // Assertion failed; native token
    let msg = ExecuteMsg::AssertMinimumReceive {
        asset_info: AssetInfo::Token {
            contract_addr: Addr::unchecked("token0000"),
        },
        prev_balance: Uint128::zero(),
        minimum_receive: Uint128::from(1000001u128),
        receiver: String::from("addr0000"),
    };
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(
        res,
        ContractError::AssertionMinimumReceive {
            receive: Uint128::new(1000001),
            amount: Uint128::new(1000000),
        }
    );
}

#[test]
fn assert_maximum_receive_swap_operations() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        ura_factory: String::from("urafactory"),
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    // We can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    let msg = ExecuteMsg::ExecuteSwapOperations {
        operations: vec![
            SwapOperation {
                offer_asset_info: AssetInfo::NativeToken {
                    denom: "ukrw".to_string(),
                },
                ask_asset_info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("asset0001"),
                },
            };
            MAX_SWAP_OPERATIONS + 1
        ],
        to: None,
        minimum_receive: None,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();

    assert_eq!(res, ContractError::SwapLimitExceeded {});
}