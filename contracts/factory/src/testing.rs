use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, from_json, to_json_binary, Addr, Reply, ReplyOn, SubMsg, SubMsgResponse, SubMsgResult,
    WasmMsg,
};
use prost::Message;
use ura::contracts::factory::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, PairConfig, PairType, PairsResponse, QueryMsg,
};
use ura::contracts::pair::InstantiateMsg as PairInstantiateMsg;
use ura::structs::asset_info::AssetInfo;
use ura::structs::pair_info::PairInfo;

use crate::contract::reply;
use crate::executes::execute;
use crate::mock_querier::mock_dependencies;
use crate::queries::query;
use crate::state::CONFIG;
use crate::{contract::instantiate, error::ContractError};

#[derive(Clone, PartialEq, Message)]
struct MsgInstantiateContractResponse {
    #[prost(string, tag = "1")]
    pub contract_address: String,
    #[prost(bytes, tag = "2")]
    pub data: Vec<u8>,
}

#[test]
fn pair_type_to_string() {
    assert_eq!(PairType::Xyk.to_string(), "xyk");
    assert_eq!(PairType::Stable.to_string(), "stable");
}

#[test]
fn proper_initialization() {
    // Validate total fee bps
    let mut deps = mock_dependencies(&[]);
    let owner = "owner0000".to_string();

    let msg = InstantiateMsg {
        pair_configs: vec![
            PairConfig {
                code_id: 123u64,
                pair_type: PairType::Xyk,
                total_fee_bps: 100,
                is_disabled: false,
                is_controller_disabled: false,
            },
            PairConfig {
                code_id: 325u64,
                pair_type: PairType::Xyk,
                total_fee_bps: 100,
                is_disabled: false,
                is_controller_disabled: false,
            },
        ],
        controller_address: Some(String::from("controller")),
        owner: owner.clone(),
        coin_registry_address: "coin_registry".to_string(),
        fee_address: None,
        token_code_id: 123u64,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(res, ContractError::PairConfigDuplicate {});

    let msg = InstantiateMsg {
        pair_configs: vec![PairConfig {
            code_id: 123u64,
            pair_type: PairType::Xyk,
            total_fee_bps: 10_001,
            is_disabled: false,
            is_controller_disabled: false,
        }],
        controller_address: Some(String::from("controller")),
        owner: owner.clone(),
        coin_registry_address: "coin_registry".to_string(),
        fee_address: None,
        token_code_id: 123u64,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(res, ContractError::PairConfigInvalidFeeBps {});

    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        pair_configs: vec![
            PairConfig {
                code_id: 325u64,
                pair_type: PairType::Stable,
                total_fee_bps: 100,
                is_disabled: false,
                is_controller_disabled: false,
            },
            PairConfig {
                code_id: 123u64,
                pair_type: PairType::Xyk,
                total_fee_bps: 100,
                is_disabled: false,
                is_controller_disabled: false,
            },
        ],
        controller_address: Some(String::from("controller")),
        owner: owner.clone(),
        coin_registry_address: "coin_registry".to_string(),
        fee_address: None,
        token_code_id: 123u64,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    instantiate(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

    let query_res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_json(&query_res).unwrap();
    assert_eq!(msg.pair_configs, config_res.pair_configs);
    assert_eq!(Addr::unchecked(owner), config_res.owner);
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);
    let owner = "owner0000";

    let pair_configs = vec![PairConfig {
        code_id: 123u64,
        pair_type: PairType::Xyk,
        total_fee_bps: 3,
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

    let env = mock_env();
    let info = mock_info(owner, &[]);

    // We can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    // Update config
    let env = mock_env();
    let info = mock_info(owner, &[]);
    let msg = ExecuteMsg::UpdateConfig {
        fee_address: None,
        controller_address: Some(String::from("new_controller_addr")),
        coin_registry_address: None,
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // It worked, let's query the state
    let query_res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_json(&query_res).unwrap();
    assert_eq!(owner, config_res.owner);
    assert_eq!(
        String::from("new_controller_addr"),
        config_res.controller_address.unwrap()
    );

    // Unauthorized err
    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        fee_address: None,
        controller_address: None,
        coin_registry_address: None,
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {});
}

#[test]
fn update_owner() {
    let mut deps = mock_dependencies(&[]);
    let owner = "owner0000";

    let msg = InstantiateMsg {
        pair_configs: vec![],
        owner: owner.to_string(),
        controller_address: Some(String::from("controller")),
        coin_registry_address: "coin_registry".to_string(),
        fee_address: None,
        token_code_id: 123u64,
    };

    let env = mock_env();
    let info = mock_info(owner, &[]);

    // We can just call .unwrap() to assert this was a success
    instantiate(deps.as_mut(), env, info, msg).unwrap();

    let new_owner = String::from("new_owner");

    // New owner
    let env = mock_env();
    let msg = ExecuteMsg::ProposeNewOwner {
        owner: new_owner.clone(),
        expires_in: 100, // seconds
    };

    let info = mock_info(new_owner.as_str(), &[]);

    // Unauthorized check
    let err = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap_err();
    assert_eq!(err.to_string(), "Generic error: Unauthorized");

    // Claim before proposal
    let info = mock_info(new_owner.as_str(), &[]);
    execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::ClaimOwnership {},
    )
    .unwrap_err();

    // Propose new owner
    let info = mock_info(owner, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Unauthorized ownership claim
    let info = mock_info("invalid_addr", &[]);
    let err = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::ClaimOwnership {},
    )
    .unwrap_err();
    assert_eq!(err.to_string(), "Generic error: Unauthorized");

    // Claim ownership
    let info = mock_info(new_owner.as_str(), &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::ClaimOwnership {},
    )
    .unwrap();
    assert_eq!(0, res.messages.len());

    // Let's query the state
    let config: ConfigResponse =
        from_json(&query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(new_owner, config.owner);
}

#[test]
fn update_pair_config() {
    let mut deps = mock_dependencies(&[]);
    let owner = "owner0000";
    let pair_configs = vec![PairConfig {
        code_id: 123u64,
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

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    // We can just call .unwrap() to assert this was a success
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // It worked, let's query the state
    let query_res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_json(&query_res).unwrap();
    assert_eq!(pair_configs, config_res.pair_configs);

    // Update config
    let pair_config = PairConfig {
        code_id: 800,
        pair_type: PairType::Xyk,
        total_fee_bps: 1,
        is_disabled: false,
        is_controller_disabled: false,
    };

    // Unauthorized err
    let env = mock_env();
    let info = mock_info("wrong-addr0000", &[]);
    let msg = ExecuteMsg::UpdatePairConfig {
        config: pair_config.clone(),
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {});

    let info = mock_info(owner.clone(), &[]);
    let msg = ExecuteMsg::UpdatePairConfig {
        config: pair_config.clone(),
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // It worked, let's query the state
    let query_res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_json(&query_res).unwrap();
    assert_eq!(vec![pair_config.clone()], config_res.pair_configs);

    // Add second config
    let pair_config_2 = PairConfig {
        code_id: 100,
        pair_type: PairType::Stable,
        total_fee_bps: 10,
        is_disabled: false,
        is_controller_disabled: false,
    };

    let info = mock_info(owner.clone(), &[]);
    let msg = ExecuteMsg::UpdatePairConfig {
        config: pair_config_2.clone(),
    };

    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // It worked, let's query the state
    let query_res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
    let config_res: ConfigResponse = from_json(&query_res).unwrap();
    assert_eq!(
        vec![pair_config_2.clone(), pair_config.clone()],
        config_res.pair_configs
    );
}

#[test]
fn create_pair() {
    let mut deps = mock_dependencies(&[]);

    let pair_config = PairConfig {
        code_id: 321u64,
        pair_type: PairType::Xyk,
        total_fee_bps: 100,
        is_disabled: false,
        is_controller_disabled: false,
    };

    let msg = InstantiateMsg {
        pair_configs: vec![pair_config.clone()],
        owner: "owner0000".to_string(),
        controller_address: Some(String::from("controller")),
        coin_registry_address: "coin_registry".to_string(),
        fee_address: None,
        token_code_id: 123u64,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    // We can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();

    let asset_infos = vec![
        AssetInfo::Token {
            contract_addr: Addr::unchecked("asset0000"),
        },
        AssetInfo::Token {
            contract_addr: Addr::unchecked("asset0001"),
        },
    ];

    let config = CONFIG.load(&deps.storage);
    let env = mock_env();
    let info = mock_info("addr0000", &[]);

    // Check pair creation using a non-whitelisted pair ID
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::CreatePair {
            pair_type: PairType::Stable,
            asset_infos: asset_infos.clone(),
            init_params: None,
            toggle_cw20_token: Some(true),
        },
    )
    .unwrap_err();
    assert_eq!(res, ContractError::PairConfigNotFound {});

    let res = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::CreatePair {
            pair_type: PairType::Xyk,
            asset_infos: asset_infos.clone(),
            init_params: None,
            toggle_cw20_token: Some(true),
        },
    )
    .unwrap();

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "create_pair"),
            attr("pair", "asset0000-asset0001")
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg {
            msg: WasmMsg::Instantiate {
                msg: to_json_binary(&PairInstantiateMsg {
                    factory_addr: String::from(MOCK_CONTRACT_ADDR),
                    asset_infos: asset_infos.clone(),
                    init_params: None,
                    token_code_id: Some(123u64),
                })
                .unwrap(),
                code_id: pair_config.code_id,
                funds: vec![],
                admin: Some(config.unwrap().owner.to_string()),
                label: String::from("Ura Pair"),
            }
            .into(),
            id: 1,
            gas_limit: None,
            reply_on: ReplyOn::Success
        }]
    );
}

#[test]
fn register() {
    let mut deps = mock_dependencies(&[]);
    let owner = "owner0000";

    let msg = InstantiateMsg {
        pair_configs: vec![PairConfig {
            code_id: 123u64,
            pair_type: PairType::Xyk,
            total_fee_bps: 100,
            is_disabled: false,
            is_controller_disabled: false,
        }],
        controller_address: Some(String::from("controller")),
        owner: owner.to_string(),
        coin_registry_address: "coin_registry".to_string(),
        fee_address: None,
        token_code_id: 123u64,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    let asset_infos = vec![
        AssetInfo::Token {
            contract_addr: Addr::unchecked("asset0000"),
        },
        AssetInfo::Token {
            contract_addr: Addr::unchecked("asset0001"),
        },
    ];

    let msg = ExecuteMsg::CreatePair {
        pair_type: PairType::Xyk,
        asset_infos: asset_infos.clone(),
        init_params: None,
        toggle_cw20_token: None,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let pair0_addr = "pair0000".to_string();
    let pair0_info = PairInfo {
        asset_infos: asset_infos.clone(),
        contract_addr: Addr::unchecked("pair0000"),
        liquidity_token: AssetInfo::Token {
            contract_addr: Addr::unchecked("liquidity0000".to_string()),
        },
        pair_type: PairType::Xyk,
    };

    let mut deployed_pairs = vec![(&pair0_addr, &pair0_info)];

    // Register an URA pair querier
    deps.querier.with_ura_pairs(&deployed_pairs);

    let instantiate_reply = MsgInstantiateContractResponse {
        contract_address: String::from("pair0000"),
        data: vec![],
    };

    let mut encoded_instantiate_reply = Vec::<u8>::with_capacity(instantiate_reply.encoded_len());
    instantiate_reply
        .encode(&mut encoded_instantiate_reply)
        .unwrap();

    let reply_msg = Reply {
        id: 2,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(encoded_instantiate_reply.into()),
        }),
    };

    let _res = reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();

    let query_res = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Pair {
            asset_infos: asset_infos.clone(),
        },
    )
    .unwrap();

    let pair_res: PairInfo = from_json(&query_res).unwrap();
    assert_eq!(
        pair_res,
        PairInfo {
            liquidity_token: AssetInfo::Token {
                contract_addr: Addr::unchecked("liquidity0000".to_string()),
            },
            contract_addr: Addr::unchecked("pair0000"),
            asset_infos: asset_infos.clone(),
            pair_type: PairType::Xyk,
        }
    );

    // Check pair was registered
    let res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap_err();
    assert_eq!(res, ContractError::PairWasRegistered {});

    // Store one more item to test query pairs
    let asset_infos_2 = vec![
        AssetInfo::Token {
            contract_addr: Addr::unchecked("asset0000"),
        },
        AssetInfo::Token {
            contract_addr: Addr::unchecked("asset0002"),
        },
    ];

    let msg = ExecuteMsg::CreatePair {
        pair_type: PairType::Xyk,
        asset_infos: asset_infos_2.clone(),
        init_params: None,
        toggle_cw20_token: None,
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let pair1_addr = "pair0001".to_string();
    let pair1_info = PairInfo {
        asset_infos: asset_infos_2.clone(),
        contract_addr: Addr::unchecked("pair0001"),
        liquidity_token: AssetInfo::Token {
            contract_addr: Addr::unchecked("liquidity0001".to_string()),
        },
        pair_type: PairType::Xyk,
    };

    deployed_pairs.push((&pair1_addr, &pair1_info));

    // Register URA pair querier
    deps.querier.with_ura_pairs(&deployed_pairs);

    let instantiate_reply = MsgInstantiateContractResponse {
        contract_address: String::from("pair0001"),
        data: vec![],
    };

    let mut encoded_instantiate_reply = Vec::<u8>::with_capacity(instantiate_reply.encoded_len());
    instantiate_reply
        .encode(&mut encoded_instantiate_reply)
        .unwrap();

    let reply_msg = Reply {
        id: 2,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(encoded_instantiate_reply.into()),
        }),
    };

    let _res = reply(deps.as_mut(), mock_env(), reply_msg.clone()).unwrap();

    let query_msg = QueryMsg::Pairs {
        start_after: None,
        limit: None,
    };

    let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let pairs_res: PairsResponse = from_json(&res).unwrap();
    assert_eq!(
        pairs_res.pairs,
        vec![
            PairInfo {
                liquidity_token: AssetInfo::Token {
                    contract_addr: Addr::unchecked("liquidity0000".to_string()),
                },
                contract_addr: Addr::unchecked("pair0000"),
                asset_infos: asset_infos.clone(),
                pair_type: PairType::Xyk,
            },
            PairInfo {
                liquidity_token: AssetInfo::Token {
                    contract_addr: Addr::unchecked("liquidity0001".to_string()),
                },
                contract_addr: Addr::unchecked("pair0001"),
                asset_infos: asset_infos_2.clone(),
                pair_type: PairType::Xyk,
            }
        ]
    );

    let query_msg = QueryMsg::Pairs {
        start_after: None,
        limit: Some(1),
    };

    let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let pairs_res: PairsResponse = from_json(&res).unwrap();
    assert_eq!(
        pairs_res.pairs,
        vec![PairInfo {
            liquidity_token: AssetInfo::Token {
                contract_addr: Addr::unchecked("liquidity0000".to_string()),
            },
            contract_addr: Addr::unchecked("pair0000"),
            asset_infos: asset_infos.clone(),
            pair_type: PairType::Xyk,
        }]
    );

    let query_msg = QueryMsg::Pairs {
        start_after: Some(asset_infos.clone()),
        limit: None,
    };

    let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let pairs_res: PairsResponse = from_json(&res).unwrap();
    assert_eq!(
        pairs_res.pairs,
        vec![PairInfo {
            liquidity_token: AssetInfo::Token {
                contract_addr: Addr::unchecked("liquidity0001".to_string()),
            },
            contract_addr: Addr::unchecked("pair0001"),
            asset_infos: asset_infos_2.clone(),
            pair_type: PairType::Xyk,
        }]
    );

    // Deregister from wrong acc
    let env = mock_env();
    let info = mock_info("wrong_addr0000", &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::Deregister {
            asset_infos: asset_infos_2.clone(),
        },
    )
    .unwrap_err();

    assert_eq!(res, ContractError::Unauthorized {});

    // Proper deregister
    let env = mock_env();
    let info = mock_info(owner.clone(), &[]);
    let res = execute(
        deps.as_mut(),
        env.clone(),
        info,
        ExecuteMsg::Deregister {
            asset_infos: asset_infos_2.clone(),
        },
    )
    .unwrap();

    assert_eq!(res.attributes[0], attr("action", "deregister"));

    let query_msg = QueryMsg::Pairs {
        start_after: None,
        limit: None,
    };

    let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
    let pairs_res: PairsResponse = from_json(&res).unwrap();
    assert_eq!(
        pairs_res.pairs,
        vec![PairInfo {
            liquidity_token: AssetInfo::Token {
                contract_addr: Addr::unchecked("liquidity0000".to_string()),
            },
            contract_addr: Addr::unchecked("pair0000"),
            asset_infos: asset_infos.clone(),
            pair_type: PairType::Xyk,
        },]
    );
}
