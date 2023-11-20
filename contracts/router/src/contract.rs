use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;

use ura::structs::asset::Asset;
use ura::structs::asset_info::AssetInfo;
use ura::utils::validation::addr_opt_validate;

use ura::contracts::pair::{QueryMsg as PairQueryMsg, SimulationResponse};
use ura::contracts::router::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    SimulateSwapOperationsResponse, SwapOperation, MAX_SWAP_OPERATIONS,
};
use ura::utils::querier::query_pair_info;

use crate::error::ContractError;
use crate::operations::execute_swap_operation;
use crate::state::{Config, CONFIG};

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "router";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Creates a new contract with the specified parameters in the [`InstantiateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            ura_factory: deps.api.addr_validate(&msg.ura_factory)?,
        },
    )?;

    Ok(Response::default())
}

/// Exposes all the execute functions available in the contract.
///
/// ## Variants
/// * **ExecuteMsg::Receive(msg)** Receives a message of type [`Cw20ReceiveMsg`] and processes
/// it depending on the received template.
///
/// * **ExecuteMsg::ExecuteSwapOperations {
///             operations,
///             minimum_receive,
///             to
///         }** Performs swap operations with the specified parameters.
///
/// * **ExecuteMsg::ExecuteSwapOperation { operation, to }** Execute a single swap operation.
///
/// * **ExecuteMsg::AssertMinimumReceive {
///             asset_info,
///             prev_balance,
///             minimum_receive,
///             receiver
///         }** Checks if an ask amount is higher than or equal to the minimum amount to receive.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, msg),
        ExecuteMsg::ExecuteSwapOperations {
            operations,
            to,
            minimum_receive,
        } => execute_swap_operations(deps, env, info.sender, operations, to, minimum_receive),
        ExecuteMsg::ExecuteSwapOperation { operation, to } => {
            execute_swap_operation(deps, env, info, operation, to)
        }
        ExecuteMsg::AssertMinimumReceive {
            asset_info,
            prev_balance,
            minimum_receive,
            receiver,
        } => assert_minimum_receive(
            deps.as_ref(),
            asset_info,
            prev_balance,
            minimum_receive,
            deps.api.addr_validate(&receiver)?,
        ),
    }
}

/// Receives a message of type [`Cw20ReceiveMsg`] and processes it depending on the received template.
///
/// * **cw20_msg** is an object of type [`Cw20ReceiveMsg`].
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_json(&cw20_msg.msg)? {
        Cw20HookMsg::ExecuteSwapOperations {
            operations,
            to,
            minimum_receive,
        } => execute_swap_operations(
            deps,
            env,
            Addr::unchecked(cw20_msg.sender),
            operations,
            to,
            minimum_receive,
        ),
    }
}

/// Performs swap operations with the specified parameters.
///
/// * **sender** address that swaps tokens.
///
/// * **operations** all swap operations to perform.
///
/// * **minimum_receive** used to guarantee that the ask amount is above a minimum amount.
///
/// * **to** recipient of the ask tokens.
#[allow(clippy::too_many_arguments)]
pub fn execute_swap_operations(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    operations: Vec<SwapOperation>,
    to: Option<String>,
    minimum_receive: Option<Uint128>,
) -> Result<Response, ContractError> {
    assert_operations(deps.api, &operations)?;

    let to = addr_opt_validate(deps.api, &to)?.unwrap_or(sender);
    let target_asset_info = operations.last().unwrap().ask_asset_info.clone();
    let operations_len = operations.len();

    let mut messages = operations
        .into_iter()
        .enumerate()
        .map(|(operation_index, op)| {
            Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                funds: vec![],
                msg: to_json_binary(&ExecuteMsg::ExecuteSwapOperation {
                    operation: op,
                    to: if operation_index == operations_len - 1 {
                        Some(to.to_string())
                    } else {
                        None
                    },
                })?,
            }))
        })
        .collect::<StdResult<Vec<CosmosMsg>>>()?;

    // Execute minimum amount assertion
    if let Some(minimum_receive) = minimum_receive {
        let receiver_balance = target_asset_info.query_pool(&deps.querier, &to)?;
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            funds: vec![],
            msg: to_json_binary(&ExecuteMsg::AssertMinimumReceive {
                asset_info: target_asset_info,
                prev_balance: receiver_balance,
                minimum_receive,
                receiver: to.to_string(),
            })?,
        }));
    }

    Ok(Response::new().add_messages(messages))
}

/// Checks if an ask amount is equal to or above a minimum amount.
///
/// * **asset_info** asset to check the ask amount for.
///
/// * **prev_balance** previous balance that the swap receive had before getting `ask` assets.
///
/// * **minimum_receive** minimum amount of `ask` assets to receive.
///
/// * **receiver** address that received `ask` assets.
fn assert_minimum_receive(
    deps: Deps,
    asset_info: AssetInfo,
    prev_balance: Uint128,
    minimum_receive: Uint128,
    receiver: Addr,
) -> Result<Response, ContractError> {
    asset_info.check(deps.api)?;
    let receiver_balance = asset_info.query_pool(&deps.querier, receiver)?;
    let swap_amount = receiver_balance.checked_sub(prev_balance)?;

    if swap_amount < minimum_receive {
        Err(ContractError::AssertionMinimumReceive {
            receive: minimum_receive,
            amount: swap_amount,
        })
    } else {
        Ok(Response::default())
    }
}

/// Exposes all the queries available in the contract.
/// ## Queries
/// * **QueryMsg::Config {}** Returns general router parameters using a [`ConfigResponse`] object.
/// * **QueryMsg::SimulateSwapOperations {
///             offer_amount,
///             operations,
///         }** Simulates one or multiple swap operations and returns the end result in a [`SimulateSwapOperationsResponse`] object.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_json_binary(&query_config(deps)?)?),
        QueryMsg::SimulateSwapOperations {
            offer_amount,
            operations,
        } => Ok(to_json_binary(&simulate_swap_operations(
            deps,
            offer_amount,
            operations,
        )?)?),
    }
}

/// Returns general contract settings in a [`ConfigResponse`] object.
pub fn query_config(deps: Deps) -> Result<ConfigResponse, ContractError> {
    let state = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        ura_factory: state.ura_factory.into_string(),
    };

    Ok(resp)
}

/// Manages contract migration.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

/// Returns the end result of a simulation for one or multiple swap
/// operations using a [`SimulateSwapOperationsResponse`] object.
///
/// * **offer_amount** amount of offer assets being swapped.
///
/// * **operations** is a vector that contains objects of type [`SwapOperation`].
/// These are all the swap operations for which we perform a simulation.
fn simulate_swap_operations(
    deps: Deps,
    offer_amount: Uint128,
    operations: Vec<SwapOperation>,
) -> Result<SimulateSwapOperationsResponse, ContractError> {
    assert_operations(deps.api, &operations)?;

    let config = CONFIG.load(deps.storage)?;
    let ura_factory = config.ura_factory;
    let mut return_amount = offer_amount;

    for operation in operations.into_iter() {
        let offer_asset_info = operation.offer_asset_info;
        let ask_asset_info = operation.ask_asset_info;
        let pair_info = query_pair_info(
            &deps.querier,
            ura_factory.clone(),
            &[offer_asset_info.clone(), ask_asset_info.clone()],
        )?;
        let res: SimulationResponse = deps.querier.query_wasm_smart(
            pair_info.contract_addr,
            &PairQueryMsg::Simulation {
                offer_asset: Asset {
                    info: offer_asset_info.clone(),
                    amount: return_amount,
                },
                ask_asset_info: Some(ask_asset_info.clone()),
            },
        )?;

        return_amount = res.return_amount;
    }

    Ok(SimulateSwapOperationsResponse {
        amount: return_amount,
    })
}

/// Validates swap operations.
///
/// * **operations** is a vector that contains objects of type [`SwapOperation`]. These are all the swap operations we check.
fn assert_operations(api: &dyn Api, operations: &[SwapOperation]) -> Result<(), ContractError> {
    let operations_len = operations.len();
    if operations_len == 0 {
        return Err(ContractError::MustProvideOperations {});
    }

    if operations_len > MAX_SWAP_OPERATIONS {
        return Err(ContractError::SwapLimitExceeded {});
    }

    let mut prev_ask_asset: Option<AssetInfo> = None;

    for operation in operations {
        let offer_asset = operation.offer_asset_info.clone();
        offer_asset.check(api)?;
        let ask_asset = operation.ask_asset_info.clone();
        ask_asset.check(api)?;

        if offer_asset.equal(&ask_asset) {
            return Err(ContractError::DoublingAssetsPath {
                offer_asset: offer_asset.to_string(),
                ask_asset: ask_asset.to_string(),
            });
        }

        if let Some(prev_ask_asset) = prev_ask_asset {
            if prev_ask_asset != offer_asset {
                return Err(ContractError::InvalidPathOperations {
                    prev_ask_asset: prev_ask_asset.to_string(),
                    next_offer_asset: offer_asset.to_string(),
                    next_ask_asset: ask_asset.to_string(),
                });
            }
        }

        prev_ask_asset = Some(ask_asset);
    }

    Ok(())
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn test_invalid_operations() {
        use cosmwasm_std::testing::mock_dependencies;
        let deps = mock_dependencies();
        // Empty error
        assert_eq!(true, assert_operations(deps.as_ref().api, &[]).is_err());

        // uluna output
        assert_eq!(
            true,
            assert_operations(
                deps.as_ref().api,
                &vec![
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
                ]
            )
            .is_ok()
        );

        // asset0002 output
        assert_eq!(
            true,
            assert_operations(
                deps.as_ref().api,
                &vec![
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
                ]
            )
            .is_ok()
        );

        // Multiple output token type errors
        assert_eq!(
            true,
            assert_operations(
                deps.as_ref().api,
                &vec![
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
                            denom: "uaud".to_string(),
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
                ]
            )
            .is_err()
        );
    }
}
