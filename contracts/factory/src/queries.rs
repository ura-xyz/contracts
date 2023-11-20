#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Order, StdResult};
use ura::contracts::factory::{ConfigResponse, FeeInfoResponse, PairType, PairsResponse, QueryMsg};
use ura::contracts::pair::QueryMsg as PairQueryMsg;
use ura::structs::asset_info::AssetInfo;
use ura::structs::pair_info::PairInfo;

use crate::state::CREATED_PAIRS;
use crate::state::{pair_key, read_pairs, CONFIG, PAIRS, PAIR_CONFIGS};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Pair { asset_infos } => to_json_binary(&query_pair(deps, asset_infos)?),
        QueryMsg::Pairs { start_after, limit } => {
            to_json_binary(&query_pairs(deps, start_after, limit)?)
        }
        QueryMsg::FeeInfo { pair_type } => to_json_binary(&query_fee_info(deps, pair_type)?),
        QueryMsg::BlacklistedPairTypes {} => to_json_binary(&query_blacklisted_pair_types(deps)?),
    }
}

pub fn query_blacklisted_pair_types(deps: Deps) -> StdResult<Vec<PairType>> {
    PAIR_CONFIGS
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|result| match result {
            Ok(v) => {
                if v.1.is_disabled || v.1.is_controller_disabled {
                    Some(Ok(v.1.pair_type))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(e)),
        })
        .collect()
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        owner: config.owner,
        pair_configs: PAIR_CONFIGS
            .range(deps.storage, None, None, Order::Ascending)
            .map(|item| Ok(item?.1))
            .collect::<StdResult<Vec<_>>>()?,
        controller_address: config.controller_address,
        coin_registry_address: config.coin_registry_address,
        fee_address: config.fee_address,
    };

    Ok(resp)
}

pub fn query_pair(deps: Deps, asset_infos: Vec<AssetInfo>) -> StdResult<PairInfo> {
    let pair_addr = PAIRS.load(deps.storage, &pair_key(&asset_infos))?;
    query_pair_info(deps, pair_addr)
}

pub fn query_pairs(
    deps: Deps,
    start_after: Option<Vec<AssetInfo>>,
    limit: Option<u32>,
) -> StdResult<PairsResponse> {
    let pairs = read_pairs(deps, start_after, limit)?
        .iter()
        .map(|pair_addr| query_pair_info(deps, pair_addr))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(PairsResponse { pairs })
}

pub fn query_fee_info(deps: Deps, pair_type: PairType) -> StdResult<FeeInfoResponse> {
    let pair_config = PAIR_CONFIGS.load(deps.storage, pair_type.to_string())?;

    Ok(FeeInfoResponse {
        total_fee_bps: pair_config.total_fee_bps,
    })
}

/// Returns information about a pair (using the [`PairInfo`] struct).
///
/// `pair_contract` is the pair for which to retrieve information.
pub fn query_pair_info(deps: Deps, pair_contract: impl Into<String>) -> StdResult<PairInfo> {
    let pair_address = deps.api.addr_validate(&pair_contract.into())?;
    CREATED_PAIRS.load(deps.storage, &pair_address)?;
    deps.querier
        .query_wasm_smart(pair_address.to_string(), &PairQueryMsg::Pair {})
}
