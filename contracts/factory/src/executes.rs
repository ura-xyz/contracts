use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_json_binary, Binary, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response,
    StdError, SubMsg, WasmMsg,
};
use itertools::Itertools;
use ura::contracts::factory::{ExecuteMsg, PairConfig, PairType};
use ura::contracts::pair::InstantiateMsg as PairInstantiateMsg;
use ura::structs::asset_info::AssetInfo;
use ura::utils::ownership::{claim_ownership, drop_ownership_proposal, propose_new_owner};

use crate::error::ContractError;
use crate::state::{
    check_asset_infos, pair_key, TmpPairInfo, CONFIG, OWNERSHIP_PROPOSAL, PAIRS, PAIR_CONFIGS,
    TMP_PAIR_INFO,
};

pub struct UpdateConfig {
    fee_address: Option<String>,
    controller_address: Option<String>,
    coin_registry_address: Option<String>,
}

const INSTANTIATE_PAIR_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            fee_address,
            controller_address,
            coin_registry_address,
        } => execute_update_config(
            deps,
            info,
            UpdateConfig {
                fee_address,
                controller_address,
                coin_registry_address,
            },
        ),
        ExecuteMsg::UpdatePairConfig { config } => execute_update_pair_config(deps, info, config),
        ExecuteMsg::CreatePair {
            pair_type,
            asset_infos,
            init_params,
            toggle_cw20_token,
        } => execute_create_pair(
            deps,
            env,
            info,
            pair_type,
            asset_infos,
            init_params,
            toggle_cw20_token,
        ),
        ExecuteMsg::Deregister { asset_infos } => deregister(deps, info, asset_infos),
        ExecuteMsg::ProposeNewOwner { owner, expires_in } => {
            let config = CONFIG.load(deps.storage)?;

            propose_new_owner(
                deps,
                info,
                env,
                owner,
                expires_in,
                config.owner,
                OWNERSHIP_PROPOSAL,
            )
            .map_err(Into::into)
        }
        ExecuteMsg::DropOwnershipProposal {} => {
            let config = CONFIG.load(deps.storage)?;

            drop_ownership_proposal(deps, info, config.owner, OWNERSHIP_PROPOSAL)
                .map_err(Into::into)
        }
        ExecuteMsg::ClaimOwnership {} => {
            claim_ownership(deps, info, env, OWNERSHIP_PROPOSAL, |deps, new_owner| {
                CONFIG
                    .update::<_, StdError>(deps.storage, |mut v| {
                        v.owner = new_owner;
                        Ok(v)
                    })
                    .map(|_| ())
            })
            .map_err(Into::into)
        }
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    param: UpdateConfig,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // Permission check
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(controller_address) = param.controller_address {
        // Validate the address format
        config.controller_address = Some(deps.api.addr_validate(&controller_address)?);
    } else {
        // Always provide controller_address else it will be set to None
        config.controller_address = None;
    }

    if let Some(coin_registry_address) = param.coin_registry_address {
        config.coin_registry_address = deps.api.addr_validate(&coin_registry_address)?;
    }

    if let Some(fee_address) = param.fee_address {
        // Validate the address format
        config.fee_address = deps.api.addr_validate(&fee_address)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn execute_update_pair_config(
    deps: DepsMut,
    info: MessageInfo,
    pair_config: PairConfig,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Permission check
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // Validate total  fee bps
    if !pair_config.valid_fee_bps() {
        return Err(ContractError::PairConfigInvalidFeeBps {});
    }

    PAIR_CONFIGS.save(
        deps.storage,
        pair_config.pair_type.to_string(),
        &pair_config,
    )?;

    Ok(Response::new().add_attribute("action", "update_pair_config"))
}

pub fn execute_create_pair(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pair_type: PairType,
    asset_infos: Vec<AssetInfo>,
    init_params: Option<Binary>,
    toggle_cw20_token: Option<bool>,
) -> Result<Response, ContractError> {
    check_asset_infos(deps.api, &asset_infos)?;

    let config = CONFIG.load(deps.storage)?;

    if PAIRS.has(deps.storage, &pair_key(&asset_infos)) {
        return Err(ContractError::PairWasCreated {});
    }

    // Get pair type from config
    let pair_config = PAIR_CONFIGS
        .load(deps.storage, pair_type.to_string())
        .map_err(|_| ContractError::PairConfigNotFound {})?;

    // Check if pair config is disabled
    if pair_config.is_disabled {
        return Err(ContractError::PairConfigDisabled {});
    }

    let pair_key = pair_key(&asset_infos);
    TMP_PAIR_INFO.save(deps.storage, &TmpPairInfo { pair_key })?;

    let sub_msg: Vec<SubMsg> = vec![SubMsg {
        id: INSTANTIATE_PAIR_REPLY_ID,
        msg: WasmMsg::Instantiate {
            admin: Some(config.owner.to_string()),
            code_id: pair_config.code_id,
            msg: to_json_binary(&PairInstantiateMsg {
                asset_infos: asset_infos.clone(),
                factory_addr: env.contract.address.to_string(),
                init_params,
                token_code_id: if toggle_cw20_token.unwrap_or(false) {
                    Some(config.token_code_id)
                } else {
                    None
                },
            })?,
            funds: info.funds,
            label: "Ura Pair".to_string(),
        }
        .into(),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }];

    Ok(Response::new()
        .add_submessages(sub_msg)
        .add_attributes(vec![
            attr("action", "create_pair"),
            attr("pair", asset_infos.iter().join("-")),
        ]))
}

pub fn deregister(
    deps: DepsMut,
    info: MessageInfo,
    asset_infos: Vec<AssetInfo>,
) -> Result<Response, ContractError> {
    check_asset_infos(deps.api, &asset_infos)?;

    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let pair_addr = PAIRS.load(deps.storage, &pair_key(&asset_infos))?;
    PAIRS.remove(deps.storage, &pair_key(&asset_infos));

    let messages: Vec<CosmosMsg> = vec![];

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "deregister"),
        attr("pair_contract_addr", pair_addr),
    ]))
}
