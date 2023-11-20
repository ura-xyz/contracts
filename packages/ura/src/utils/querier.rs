use crate::contracts::asset_list::{AssetMetadata, QueryMsg as AssetListQueryMsg};
use crate::contracts::controller::{
    GaugeFromPoolRequest, GaugeFromPoolResponse, QueryMsg as ControllerQueryMsg,
    TotalGaugeVotesRequest, TotalGaugeVotesResponse,
};
use crate::contracts::factory::{
    Config as FactoryConfig, ConfigResponse, FeeInfoResponse, PairType, PairsResponse,
    QueryMsg as FactoryQueryMsg,
};
use crate::structs::asset_info::AssetInfo;
use crate::structs::fee_info::FeeInfo;
use crate::structs::pair_info::PairInfo;

use cosmwasm_std::{
    from_json, to_json_binary, Addr, AllBalanceResponse, BankQuery, Coin, CustomQuery, Decimal,
    QuerierWrapper, QueryRequest, StdError, StdResult, Uint128, WasmQuery,
};

use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg, TokenInfoResponse};

/// Returns a native token's balance for a specific account.
///
/// * **denom** specifies the denomination used to return the balance (e.g uluna).
pub fn query_balance<C>(
    querier: &QuerierWrapper<C>,
    account_addr: impl Into<String>,
    denom: impl Into<String>,
) -> StdResult<Uint128>
where
    C: CustomQuery,
{
    Ok(querier
        .query_balance(account_addr, denom)
        .map_or(Uint128::zero(), |coin| coin.amount))
}

/// Returns the total balances for all coins at a specified account address.
///
/// * **account_addr** address for which we query balances.
pub fn query_all_balances(querier: &QuerierWrapper, account_addr: Addr) -> StdResult<Vec<Coin>> {
    let all_balances: AllBalanceResponse =
        querier.query(&QueryRequest::Bank(BankQuery::AllBalances {
            address: String::from(account_addr),
        }))?;
    Ok(all_balances.amount)
}

/// Returns a token balance for an account.
///
/// * **contract_addr** token contract for which we return a balance.
///
/// * **account_addr** account address for which we return a balance.
pub fn query_token_balance<C>(
    querier: &QuerierWrapper<C>,
    contract_addr: impl Into<String>,
    account_addr: impl Into<String>,
) -> StdResult<Uint128>
where
    C: CustomQuery,
{
    // load balance from the token contract
    let resp: Cw20BalanceResponse = querier
        .query_wasm_smart(
            contract_addr,
            &Cw20QueryMsg::Balance {
                address: account_addr.into(),
            },
        )
        .unwrap_or_else(|_| Cw20BalanceResponse {
            balance: Uint128::zero(),
        });

    Ok(resp.balance)
}

/// Returns a token's symbol.
///
/// * **contract_addr** token contract address.
pub fn query_token_symbol<C>(
    querier: &QuerierWrapper<C>,
    contract_addr: impl Into<String>,
) -> StdResult<String>
where
    C: CustomQuery,
{
    let res: TokenInfoResponse =
        querier.query_wasm_smart(contract_addr, &Cw20QueryMsg::TokenInfo {})?;

    Ok(res.symbol)
}

/// Returns the total supply of a specific token.
///
/// * **contract_addr** token contract address.
pub fn query_supply<C>(
    querier: &QuerierWrapper<C>,
    contract_addr: impl Into<String>,
) -> StdResult<Uint128>
where
    C: CustomQuery,
{
    let res: TokenInfoResponse =
        querier.query_wasm_smart(contract_addr, &Cw20QueryMsg::TokenInfo {})?;

    Ok(res.total_supply)
}

/// Returns the number of decimals that a token has.
///
/// * **asset_info** is an object of type [`AssetInfo`] and contains the asset details for a specific token.
pub fn query_token_precision<C>(
    querier: &QuerierWrapper<C>,
    asset_info: &AssetInfo,
    factory_addr: &Addr,
) -> StdResult<u8>
where
    C: CustomQuery,
{
    Ok(match asset_info {
        AssetInfo::NativeToken { denom } => {
            let res = query_factory_config(querier, factory_addr)?;
            let result = crate::contracts::native_coin_registry::COINS_INFO.query(
                querier,
                res.coin_registry_address,
                denom.to_string(),
            )?;

            if let Some(decimals) = result {
                decimals
            } else {
                return Err(StdError::generic_err(format!(
                    "The {denom} precision was not found"
                )));
            }
        }
        AssetInfo::Token { contract_addr } => {
            let res: TokenInfoResponse =
                querier.query_wasm_smart(contract_addr, &Cw20QueryMsg::TokenInfo {})?;

            res.decimals
        }
    })
}

/// Returns the configuration for the factory contract.
pub fn query_factory_config<C>(
    querier: &QuerierWrapper<C>,
    factory_contract: impl Into<String>,
) -> StdResult<FactoryConfig>
where
    C: CustomQuery,
{
    if let Some(res) = querier.query_wasm_raw(factory_contract, b"config".as_slice())? {
        let res = from_json(&res)?;
        Ok(res)
    } else {
        Err(StdError::generic_err("The factory config not found!"))
    }
}

/// Returns the fee information for a specific pair type.
///
/// * **pair_type** pair type we query information for.
/// * **pool_address** pool address to mapped gauge address.
pub fn query_fee_info<C>(
    querier: &QuerierWrapper<C>,
    factory_contract: &Addr,
    pair_type: PairType,
    pool_address: &Addr,
) -> StdResult<FeeInfo>
where
    C: CustomQuery,
{
    let res: ConfigResponse =
        querier.query_wasm_smart(factory_contract.clone(), &FactoryQueryMsg::Config {})?;

    let fee_address = res.fee_address;
    let mut controller_address = None;
    let mut gauge_address: Option<Addr> = None;
    if let Some(address) = res.controller_address {
        controller_address = Some(address.clone());
        let res: Result<GaugeFromPoolResponse, StdError> = querier.query_wasm_smart(
            address.clone(),
            &ControllerQueryMsg::GaugeFromPool(GaugeFromPoolRequest {
                pool: pool_address.into(),
            }),
        );
        if let Ok(gauge_res) = res {
            gauge_address = Some(gauge_res.gauge);
        }
    };

    let res: FeeInfoResponse = querier.query_wasm_smart(
        factory_contract.clone(),
        &FactoryQueryMsg::FeeInfo { pair_type },
    )?;

    Ok(FeeInfo {
        fee_address,
        controller_address,
        gauge_address,
        total_fee_rate: Decimal::from_ratio(res.total_fee_bps, 10000u16),
    })
}

/// Accepts two tokens as input and returns a pair's information.
pub fn query_pair_info(
    querier: &QuerierWrapper,
    factory_contract: impl Into<String>,
    asset_infos: &[AssetInfo],
) -> StdResult<PairInfo> {
    querier.query_wasm_smart(
        factory_contract,
        &FactoryQueryMsg::Pair {
            asset_infos: asset_infos.to_vec(),
        },
    )
}

/// Returns a vector that contains items of type [`PairInfo`] which
/// symbolize pairs instantiated in the Ura factory
pub fn query_pairs_info(
    querier: &QuerierWrapper,
    factory_contract: impl Into<String>,
    start_after: Option<Vec<AssetInfo>>,
    limit: Option<u32>,
) -> StdResult<PairsResponse> {
    querier.query_wasm_smart(
        factory_contract,
        &FactoryQueryMsg::Pairs { start_after, limit },
    )
}

pub fn query_total_votes_in_gauge(
    query: &QuerierWrapper,
    controller: String,
    gauge: String,
    at_epoch: Option<u64>,
) -> StdResult<TotalGaugeVotesResponse> {
    let res: TotalGaugeVotesResponse = query.query_wasm_smart(
        controller.to_string(),
        &ControllerQueryMsg::TotalGaugeVotes(TotalGaugeVotesRequest {
            gauge: gauge.to_string(),
            epoch: at_epoch,
        }),
    )?;
    Ok(res)
}

pub fn query_from_asset_list(
    querier: &QuerierWrapper,
    asset_list: &Addr,
    asset_info: &AssetInfo,
) -> StdResult<AssetMetadata> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: asset_list.to_string(),
        msg: to_json_binary(&AssetListQueryMsg::GetAsset(asset_info.clone()))?,
    }))
}
