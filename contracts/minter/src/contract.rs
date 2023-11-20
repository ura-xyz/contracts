use cosmwasm_std::{
    attr, entry_point, to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps,
    DepsMut, Env, MessageInfo, Response, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use std::ops::{Mul, Sub};
use ura::contracts::controller::UpdateEmissionsRequest;
use ura::contracts::minter::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, UpdateConfigRequest,
};
use ura::contracts::ve_stake::{query_total_voting_power, RebaseRequest as VeRebaseRequest};
use ura::utils::math::truncate;
use ura::utils::time::get_current_epoch;

use crate::denom::{MsgCreateDenom, MsgMint};
use crate::error::ContractError;
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let team_wallet = deps.api.addr_validate(&msg.team_wallet)?;
    let contract_addr = env.contract.address.to_string();
    let denom = format!("factory/{}/URA", contract_addr.clone());

    let base_token_msg: CosmosMsg = MsgCreateDenom {
        sender: contract_addr.clone(),
        subdenom: String::from("URA"),
    }
    .into();

    // Mint initial supply to team_wallet for initial allocation
    let mint_msg: CosmosMsg = MsgMint {
        sender: contract_addr.clone(),
        amount: Some(crate::denom::Coin {
            denom: denom.clone(),
            amount: msg.initial_supply.to_string(),
        }),
    }
    .into();
    let send_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
        to_address: team_wallet.to_string(),
        amount: vec![Coin {
            denom: denom.clone(),
            amount: msg.initial_supply,
        }],
    });

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        current_epoch: 0,
        base_token: denom,
        initial_supply: msg.initial_supply,
        epoch_length: msg.epoch_duration,
        inflation: msg.inflation,
        decay: msg.decay,
        min_inflation: msg.min_inflation,
        team_allocation: msg.team_allocation,
        team_wallet,
        // Used in phrase 2
        is_emitting: false,
        epoch_start_time: msg.epoch_start_time,
        controller: Addr::unchecked(""),
        ve_stake: Addr::unchecked(""),
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default().add_messages(vec![base_token_msg, mint_msg, send_msg]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::EndEpoch {} => end_epoch(deps, env, info),
        ExecuteMsg::SetVeStaking {} => set_ve_staking(deps, env, info),
        ExecuteMsg::SetGaugeController {} => set_gauge_controller(deps, env, info),
        ExecuteMsg::UpdateConfig(req) => update_config(deps, env, info, req),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

fn set_ve_staking(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.ve_stake != Addr::unchecked("") {
        return Err(ContractError::Unauthorized {});
    }
    // ensure that creator of the incoming contract is the same for the current contract
    let current_contract_info = deps
        .querier
        .query_wasm_contract_info(env.contract.address.to_string())?;
    let ve_stake_info = deps
        .querier
        .query_wasm_contract_info(info.sender.to_string())?;
    if current_contract_info.creator.ne(&ve_stake_info.creator) {
        return Err(ContractError::Unauthorized {});
    }

    config.ve_stake = info.sender.clone();
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attributes(vec![
        attr("action", "set_ve_staking"),
        attr("ve_staking", info.sender.to_string()),
    ]))
}

fn set_gauge_controller(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.controller != Addr::unchecked("") {
        return Err(ContractError::Unauthorized {});
    }
    // ensure that sender is creator of the current contract
    let current_contract_info = deps
        .querier
        .query_wasm_contract_info(env.contract.address.to_string())?;
    let gauge_controller_info = deps
        .querier
        .query_wasm_contract_info(info.sender.to_string())?;
    if current_contract_info
        .creator
        .ne(&gauge_controller_info.creator)
    {
        return Err(ContractError::Unauthorized {});
    }

    config.controller = info.sender.clone();
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attributes(vec![
        attr("action", "set_gauge_controller"),
        attr("gauge_controller", info.sender.to_string()),
    ]))
}

fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    req: UpdateConfigRequest,
) -> Result<Response, ContractError> {
    // ensure that sender is creator of the current contract
    let current_contract_info = deps
        .querier
        .query_wasm_contract_info(env.contract.address.to_string())?;
    if current_contract_info.creator.ne(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut config = CONFIG.load(deps.storage)?;

    if let Some(team_wallet) = req.team_wallet {
        config.team_wallet = deps.api.addr_validate(&team_wallet)?;
    }

    if !config.is_emitting {
        if let Some(epoch_length) = req.epoch_length {
            config.epoch_length = epoch_length;
        }
        if let Some(inflation) = req.inflation {
            config.inflation = inflation;
        }
        if let Some(decay) = req.decay {
            config.decay = decay;
        }
        if let Some(min_inflation) = req.min_inflation {
            config.min_inflation = min_inflation;
        }
        if let Some(team_allocation) = req.team_allocation {
            config.team_allocation = team_allocation;
        }
        if let Some(epoch_start_time) = req.epoch_start_time {
            config.epoch_start_time = epoch_start_time;
        }
        if let Some(is_emitting) = req.is_emitting {
            if config.controller == Addr::unchecked("") || config.ve_stake == Addr::unchecked("") {
                return Err(ContractError::InvalidRequest(
                    "Contract cannot be emitting without controller and ve_stake contract"
                        .to_owned(),
                ));
            }
            config.is_emitting = is_emitting;
        }
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attributes(vec![
        attr("epoch_length", config.epoch_length.to_string()),
        attr("inflation", config.inflation.to_string()),
        attr("decay", config.decay.to_string()),
        attr("min_inflation", config.min_inflation.to_string()),
        attr("team_allocation", config.team_allocation.to_string()),
        attr("epoch_start_time", config.epoch_start_time.to_string()),
        attr("team_wallet", config.team_wallet.to_string()),
        attr("is_emitting", config.is_emitting.to_string()),
    ]))
}

fn end_epoch(deps: DepsMut, env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if !config.is_emitting {
        return Err(ContractError::InvalidRequest(
            "Config is current not set for emissions.".to_string(),
        ));
    }

    let actual_epoch = get_current_epoch(
        env.block.time.seconds(),
        config.epoch_start_time,
        config.epoch_length,
    );
    if actual_epoch <= config.current_epoch {
        return Err(ContractError::CannotEndEpoch(
            (config.current_epoch + 1) * config.epoch_length,
        ));
    }
    // Calculate total emissions
    // emissions = initial_supply * inflation * (decay ^ epoch)
    // inflation has a lower bound of config.min_inflation
    let decay = (Decimal::one() - config.decay).pow(config.current_epoch as u32);
    let actual_inflation = decay
        .checked_mul(config.inflation)
        .map_err(ContractError::OverflowError)?
        .max(config.min_inflation);

    let total_emissions = actual_inflation.mul(config.initial_supply);

    // Calculate team emissions
    let team_emissions = config.team_allocation.mul(total_emissions);

    // Calculate rebase emissions

    // query total voting power
    let ve_supply = query_total_voting_power(deps.as_ref(), None, config.ve_stake.clone())?.weight;

    let token_supply = config.initial_supply;

    let rebase_emissions_dec = Decimal::from_ratio(ve_supply, token_supply)
        .pow(3)
        .mul(Decimal::from_ratio(total_emissions, Uint128::new(2)));

    let rebase_emissions = truncate(rebase_emissions_dec)?;

    // Create message to mint naked tokens
    let mut mint_msgs = mint_naked_tokens(
        rebase_emissions,
        &env.contract.address,
        &env.contract.address,
        config.base_token.clone(),
    );

    // mint LP emissions
    let lp_emissions = total_emissions.sub(team_emissions).sub(rebase_emissions);
    mint_msgs.extend(mint_naked_tokens(
        lp_emissions,
        &env.contract.address,
        &env.contract.address,
        config.base_token.clone(),
    ));
    // mint esTokens for team
    mint_msgs.extend(mint_naked_tokens(
        team_emissions,
        &env.contract.address,
        &config.team_wallet,
        config.base_token.clone(),
    ));

    // mint tokens and deposit into ve contract
    let mut msgs = vec![];
    // Create message to send naked tokens to ve-contract
    msgs.push(SubMsg::new(WasmMsg::Execute {
        contract_addr: config.ve_stake.to_string(),
        msg: to_json_binary(&VeRebaseRequest {
            for_epoch: actual_epoch,
        })?,
        funds: vec![Coin {
            denom: config.base_token.clone(),
            amount: rebase_emissions,
        }],
    }));
    
    // Create message to update controller with emissions
    msgs.push(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.controller.to_string(),
        msg: to_json_binary(&UpdateEmissionsRequest { pool: None })?,
        funds: vec![Coin {
            denom: config.base_token.clone(),
            amount: lp_emissions,
        }],
    })));

    config.current_epoch = actual_epoch;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default()
        .add_attributes(vec![
            attr("action", "end_epoch"),
            attr("new_epoch", format!("{}", config.current_epoch)),
            attr("lp_emissions", &lp_emissions.to_string()),
            attr("rebase_emissions", &rebase_emissions.to_string()),
            attr("team_emissions", &team_emissions.to_string()),
        ])
        .add_messages(mint_msgs)
        .add_submessages(msgs))
}

// Create message to mint naked tokens
fn mint_naked_tokens(
    amount: Uint128,
    sender: &Addr,
    recipient: &Addr,
    denom: String,
) -> Vec<CosmosMsg> {
    let mut messages = vec![];
    messages.push(<MsgMint as Into<CosmosMsg>>::into(MsgMint {
        sender: sender.to_string(),
        amount: Some(crate::denom::Coin {
            denom: denom.clone(),
            amount: amount.to_string(),
        }),
    }));

    if sender != recipient {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![Coin {
                denom: denom,
                amount: amount,
            }],
        }));
    }
    messages
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::TokenInfo { .. } => Ok(Binary::default()),
        QueryMsg::DownloadLogo { .. } => Ok(Binary::default()),
    }
}

fn query_config(deps: Deps) -> Result<Binary, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    to_json_binary(&config).map_err(|e| ContractError::Std(e))
}
