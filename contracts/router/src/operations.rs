use cosmwasm_std::{
    to_json_binary, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdResult,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use ura::contracts::pair::ExecuteMsg as PairExecuteMsg;
use ura::contracts::router::SwapOperation;
use ura::structs::asset::Asset;
use ura::structs::asset_info::AssetInfo;
use ura::utils::querier::{query_balance, query_pair_info, query_token_balance};

use crate::error::ContractError;
use crate::state::CONFIG;

/// Execute a swap operation.
///
/// * **operation** to perform with offer and ask asset information.
///
/// * **to** address that receives the ask assets.
pub fn execute_swap_operation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operation: SwapOperation,
    to: Option<String>,
) -> Result<Response, ContractError> {
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let config = CONFIG.load(deps.storage)?;
    let offer_asset_info = operation.offer_asset_info;
    let ask_asset_info = operation.ask_asset_info;
    let pair_info = query_pair_info(
        &deps.querier,
        config.ura_factory,
        &[offer_asset_info.clone(), ask_asset_info.clone()],
    )?;

    let amount = match &offer_asset_info {
        AssetInfo::NativeToken { denom } => {
            query_balance(&deps.querier, env.contract.address, denom)?
        }
        AssetInfo::Token { contract_addr } => {
            query_token_balance(&deps.querier, contract_addr, env.contract.address)?
        }
    };
    let offer_asset = Asset {
        info: offer_asset_info,
        amount,
    };

    let message = asset_into_swap_msg(
        pair_info.contract_addr.to_string(),
        offer_asset,
        ask_asset_info,
        to,
    )?;

    Ok(Response::new().add_message(message))
}

/// Creates a message of type [`CosmosMsg`] representing a swap operation.
///
/// * **pair_contract** Ura pair contract for which the swap operation is performed.
///
/// * **offer_asset** asset that is swapped. It also mentions the amount to swap.
///
/// * **ask_asset_info** asset that is swapped to.
///
/// * **max_spread** max spread enforced for the swap.
///
/// * **to** address that receives the ask assets.
///
/// * **single** defines whether this swap is single or part of a multi hop route.
pub fn asset_into_swap_msg(
    pair_contract: String,
    offer_asset: Asset,
    ask_asset_info: AssetInfo,
    to: Option<String>,
) -> StdResult<CosmosMsg> {
    // Disable spread assertion on underlying pair contract.
    let max_spread = Some(Decimal::one());

    match &offer_asset.info {
        AssetInfo::NativeToken { denom } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: pair_contract,
            funds: vec![Coin {
                denom: denom.to_string(),
                amount: offer_asset.amount,
            }],
            msg: to_json_binary(&PairExecuteMsg::Swap {
                offer_asset: Asset {
                    amount: offer_asset.amount,
                    ..offer_asset
                },
                ask_asset_info: Some(ask_asset_info),
                belief_price: None,
                max_spread,
                to,
            })?,
        })),
        AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            funds: vec![],
            msg: to_json_binary(&Cw20ExecuteMsg::Send {
                contract: pair_contract,
                amount: offer_asset.amount,
                msg: to_json_binary(&ura::contracts::pair::Cw20HookMsg::Swap {
                    ask_asset_info: Some(ask_asset_info),
                    belief_price: None,
                    max_spread,
                    to,
                })?,
            })?,
        })),
    }
}
