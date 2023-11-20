use cosmwasm_std::{
    to_json_binary, Addr, Api, BankMsg, Coin, CosmosMsg, Decimal, Env, QuerierWrapper, StdResult,
    Storage, Uint128, Uint64, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use itertools::Itertools;
use std::cmp::Ordering;

use ura::structs::asset::Asset;
use ura::structs::asset_info::AssetInfo;
use ura::structs::decimal256::Decimal256Ext;
use ura::structs::decimal256_asset::Decimal256Asset;

use crate::denom::MsgMint;
use crate::error::ContractError;
use crate::math::calc_y;
use crate::state::{get_precision, Config};

/// Helper function to check if the given asset infos are valid.
pub(crate) fn check_asset_infos(
    api: &dyn Api,
    asset_infos: &[AssetInfo],
) -> Result<(), ContractError> {
    if !asset_infos.iter().all_unique() {
        return Err(ContractError::DoublingAssets {});
    }

    asset_infos
        .iter()
        .try_for_each(|asset_info| asset_info.check(api))
        .map_err(Into::into)
}

/// Helper function to check that the assets in a given array are valid.
pub(crate) fn check_assets(api: &dyn Api, assets: &[Asset]) -> Result<(), ContractError> {
    let asset_infos = assets.iter().map(|asset| asset.info.clone()).collect_vec();
    check_asset_infos(api, &asset_infos)
}

/// Checks that cw20 token is part of the pool.
///
/// * **cw20_sender** is cw20 token address which is being checked.
pub(crate) fn check_cw20_in_pool(config: &Config, cw20_sender: &Addr) -> Result<(), ContractError> {
    for asset_info in &config.pair_info.asset_infos {
        match asset_info {
            AssetInfo::Token { contract_addr } if contract_addr == cw20_sender => return Ok(()),
            _ => {}
        }
    }

    Err(ContractError::Unauthorized {})
}

/// Select offer and ask pools based on given offer and ask infos.
/// This function works with pools with up to 5 assets. Returns (offer_pool, ask_pool) in case of success.
/// If it is impossible to define offer and ask pools, returns [`ContractError`].
///
/// * **offer_asset_info** - asset info of the offer asset.
///
/// * **ask_asset_info** - asset info of the ask asset.
///
/// * **pools** - list of pools.
pub(crate) fn select_pools(
    offer_asset_info: Option<&AssetInfo>,
    ask_asset_info: Option<&AssetInfo>,
    pools: &[Decimal256Asset],
) -> Result<(Decimal256Asset, Decimal256Asset), ContractError> {
    if pools.len() == 2 {
        match (offer_asset_info, ask_asset_info) {
            (Some(offer_asset_info), _) => {
                let (offer_ind, offer_pool) = pools
                    .iter()
                    .find_position(|pool| pool.info.eq(offer_asset_info))
                    .ok_or(ContractError::AssetMismatch {})?;
                Ok((offer_pool.clone(), pools[(offer_ind + 1) % 2].clone()))
            }
            (_, Some(ask_asset_info)) => {
                let (ask_ind, ask_pool) = pools
                    .iter()
                    .find_position(|pool| pool.info.eq(ask_asset_info))
                    .ok_or(ContractError::AssetMismatch {})?;
                Ok((pools[(ask_ind + 1) % 2].clone(), ask_pool.clone()))
            }
            _ => Err(ContractError::VariableAssetMissed {}), // Should always be unreachable
        }
    } else if let (Some(offer_asset_info), Some(ask_asset_info)) =
        (offer_asset_info, ask_asset_info)
    {
        if ask_asset_info.eq(offer_asset_info) {
            return Err(ContractError::SameAssets {});
        }

        let offer_pool = pools
            .iter()
            .find(|pool| pool.info.eq(offer_asset_info))
            .ok_or(ContractError::AssetMismatch {})?;
        let ask_pool = pools
            .iter()
            .find(|pool| pool.info.eq(ask_asset_info))
            .ok_or(ContractError::AssetMismatch {})?;

        Ok((offer_pool.clone(), ask_pool.clone()))
    } else {
        Err(ContractError::VariableAssetMissed {}) // Should always be unreachable
    }
}

/// Compute the current pool amplification coefficient (AMP).
pub fn compute_current_amp(config: &Config, env: &Env) -> StdResult<Uint64> {
    let block_time = env.block.time.seconds();
    if block_time < config.next_amp_time {
        let elapsed_time: Uint128 = block_time.saturating_sub(config.init_amp_time).into();
        let time_range = config
            .next_amp_time
            .saturating_sub(config.init_amp_time)
            .into();
        let init_amp = Uint128::from(config.init_amp);
        let next_amp = Uint128::from(config.next_amp);

        if next_amp > init_amp {
            let amp_range = next_amp - init_amp;
            let res = init_amp + (amp_range * elapsed_time).checked_div(time_range)?;
            Ok(res.try_into()?)
        } else {
            let amp_range = init_amp - next_amp;
            let res = init_amp - (amp_range * elapsed_time).checked_div(time_range)?;
            Ok(res.try_into()?)
        }
    } else {
        Ok(Uint64::from(config.next_amp))
    }
}

/// Returns a value using a newly specified precision.
///
/// * **value** value that will have its precision adjusted.
///
/// * **current_precision** `value`'s current precision
///
/// * **new_precision** new precision to use when returning the `value`.
pub(crate) fn adjust_precision(
    value: Uint128,
    current_precision: u8,
    new_precision: u8,
) -> StdResult<Uint128> {
    Ok(match current_precision.cmp(&new_precision) {
        Ordering::Equal => value,
        Ordering::Less => value.checked_mul(Uint128::new(
            10_u128.pow((new_precision - current_precision) as u32),
        ))?,
        Ordering::Greater => value.checked_div(Uint128::new(
            10_u128.pow((current_precision - new_precision) as u32),
        ))?,
    })
}

/// Mint LP tokens for a beneficiary and auto stake the tokens in the Controller contract (if auto staking is specified).
///
/// * **recipient** LP token recipient.
///
/// * **amount** amount of LP tokens that will be minted for the recipient.
///
/// * **auto_stake** whether the newly minted LP tokens will be automatically staked in the Controller on behalf of the recipient.
pub(crate) fn mint_liquidity_token_message(
    _querier: QuerierWrapper,
    config: &Config,
    contract_address: &Addr,
    recipient: &Addr,
    amount: Uint128,
) -> Result<Vec<CosmosMsg>, ContractError> {
    match &config.pair_info.liquidity_token {
        AssetInfo::NativeToken { denom } => {
            if recipient == contract_address {
                return Ok(vec![MsgMint {
                    sender: contract_address.to_string(),
                    amount: Some(crate::denom::Coin {
                        denom: denom.clone(),
                        amount: amount.to_string(),
                    }),
                }
                .into()]);
            }

            return Ok(vec![
                MsgMint {
                    sender: contract_address.to_string(),
                    amount: Some(crate::denom::Coin {
                        denom: denom.clone(),
                        amount: amount.to_string(),
                    }),
                }
                .into(),
                CosmosMsg::Bank(BankMsg::Send {
                    to_address: recipient.to_string(),
                    amount: vec![Coin {
                        denom: denom.clone(),
                        amount: amount,
                    }],
                }),
            ]);
        }
        AssetInfo::Token { contract_addr } => {
            return Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::Mint {
                    recipient: recipient.to_string(),
                    amount,
                })?,
                funds: vec![],
            })]);
        }
    };
}

/// Return the amount of tokens that a specific amount of LP tokens would withdraw.
///
/// * **pools** array with assets available in the pool.
///
/// * **amount** amount of LP tokens to calculate underlying amounts for.
///
/// * **total_share** total amount of LP tokens currently issued by the pool.
pub(crate) fn get_share_in_assets(
    pools: &[Asset],
    amount: Uint128,
    total_share: Uint128,
) -> Vec<Asset> {
    let mut share_ratio = Decimal::zero();
    if !total_share.is_zero() {
        share_ratio = Decimal::from_ratio(amount, total_share);
    }

    pools
        .iter()
        .map(|pool| Asset {
            info: pool.info.clone(),
            amount: pool.amount * share_ratio,
        })
        .collect()
}

/// Structure for internal use which represents swap result.
pub(crate) struct SwapResult {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
}

/// Returns the result of a swap in form of a [`SwapResult`] object.
///
/// * **offer_asset** asset that is being offered.
///
/// * **offer_pool** pool of offered asset.
///
/// * **ask_pool** asked asset.
///
/// * **pools** array with assets available in the pool.
pub(crate) fn compute_swap(
    storage: &dyn Storage,
    env: &Env,
    config: &Config,
    offer_asset: &Decimal256Asset,
    offer_pool: &Decimal256Asset,
    ask_pool: &Decimal256Asset,
    pools: &[Decimal256Asset],
) -> Result<SwapResult, ContractError> {
    let token_precision = get_precision(storage, &ask_pool.info)?;
    let xp = pools.iter().map(|p| p.amount).collect_vec();

    let new_ask_pool = calc_y(
        compute_current_amp(config, env)?,
        offer_pool.amount + offer_asset.amount,
        &xp,
        token_precision,
    )?;

    let return_amount = ask_pool.amount.to_uint128_with_precision(token_precision)? - new_ask_pool;
    let offer_asset_amount = offer_asset
        .amount
        .to_uint128_with_precision(token_precision)?;

    // We consider swap rate 1:1 in stable swap thus any difference is considered as spread.
    let spread_amount = offer_asset_amount.saturating_sub(return_amount);

    Ok(SwapResult {
        return_amount,
        spread_amount,
    })
}
