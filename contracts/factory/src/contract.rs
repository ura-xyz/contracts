use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdError, SubMsgResponse, SubMsgResult,
};
use cw2::{get_contract_version, set_contract_version};
use cw_utils::parse_instantiate_response_data;
use std::collections::HashSet;
use ura::contracts::factory::{Config, InstantiateMsg, MigrateMsg};
use ura::utils::validation::addr_opt_validate;

use crate::error::ContractError;
use crate::state::{CONFIG, CREATED_PAIRS, PAIRS, PAIR_CONFIGS, TMP_PAIR_INFO};

const CONTRACT_NAME: &str = "pair-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let fee_accumulation_address = if let Some(fee_address) = msg.fee_address {
        deps.api.addr_validate(&fee_address)?
    } else {
        info.sender
    };

    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        controller_address: addr_opt_validate(deps.api, &msg.controller_address)?,
        fee_address: fee_accumulation_address,
        coin_registry_address: deps.api.addr_validate(&msg.coin_registry_address)?,
        token_code_id: msg.token_code_id,
    };

    let config_set: HashSet<String> = msg
        .pair_configs
        .iter()
        .map(|pc| pc.pair_type.to_string())
        .collect();

    if config_set.len() != msg.pair_configs.len() {
        return Err(ContractError::PairConfigDuplicate {});
    }

    for pc in msg.pair_configs.iter() {
        // Validate total fee bps
        if !pc.valid_fee_bps() {
            return Err(ContractError::PairConfigInvalidFeeBps {});
        }
        PAIR_CONFIGS.save(deps.storage, pc.pair_type.to_string(), pc)?;
    }
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg {
        Reply {
            id: _,
            result:
                SubMsgResult::Ok(SubMsgResponse {
                    data: Some(data), ..
                }),
        } => {
            let tmp = TMP_PAIR_INFO.load(deps.storage)?;
            if PAIRS.has(deps.storage, &tmp.pair_key) {
                return Err(ContractError::PairWasRegistered {});
            }

            let init_response = parse_instantiate_response_data(data.as_slice())
                .map_err(|e| StdError::generic_err(format!("{e}")))?;

            let pair_contract = deps.api.addr_validate(&init_response.contract_address)?;
            CREATED_PAIRS.save(deps.storage, &pair_contract, &Empty {})?;

            PAIRS.save(deps.storage, &tmp.pair_key, &pair_contract)?;

            Ok(Response::new().add_attributes(vec![
                attr("action", "register"),
                attr("pair_contract_addr", pair_contract),
            ]))
        }
        _ => Err(ContractError::FailedToParseReply {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new()
        .add_attribute("previous_contract_name", &contract_version.contract)
        .add_attribute("previous_contract_version", &contract_version.version)
        .add_attribute("new_contract_name", CONTRACT_NAME)
        .add_attribute("new_contract_version", CONTRACT_VERSION))
}
