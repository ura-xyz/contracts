use super::asset::Asset;
use super::asset_info::AssetInfo;
use super::decimal256_asset::Decimal256Asset;
use crate::contracts::factory::PairType;
use crate::contracts::pair::QueryMsg as PairQueryMsg;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CustomQuery, Decimal256, QuerierWrapper, StdError, StdResult};
use cw20::{Cw20QueryMsg, MinterResponse};

/// This structure stores the main parameters for an Ura pair
#[cw_serde]
pub struct PairInfo {
    /// Asset information for the assets in the pool
    pub asset_infos: Vec<AssetInfo>,
    /// Pair contract address
    pub contract_addr: Addr,
    /// Pair LP token denom
    pub liquidity_token: AssetInfo,
    /// The pool type (xyk, stableswap etc) available in [`PairType`]
    pub pair_type: PairType,
}

impl PairInfo {
    /// Returns the balance for each asset in the pool.
    ///
    /// * **contract_addr** is pair's pool address.
    pub fn query_pools<C>(
        &self,
        querier: &QuerierWrapper<C>,
        contract_addr: impl Into<String>,
    ) -> StdResult<Vec<Asset>>
    where
        C: CustomQuery,
    {
        let contract_addr = contract_addr.into();
        self.asset_infos
            .iter()
            .map(|asset_info| {
                Ok(Asset {
                    info: asset_info.clone(),
                    amount: asset_info.query_pool(querier, &contract_addr)?,
                })
            })
            .collect()
    }

    /// Returns the balance for each asset in the pool in decimal.
    ///
    /// * **contract_addr** is pair's pool address.
    pub fn query_pools_decimal(
        &self,
        querier: &QuerierWrapper,
        contract_addr: impl Into<String>,
        factory_addr: &Addr,
    ) -> StdResult<Vec<Decimal256Asset>> {
        let contract_addr = contract_addr.into();
        self.asset_infos
            .iter()
            .map(|asset_info| {
                Ok(Decimal256Asset {
                    info: asset_info.clone(),
                    amount: Decimal256::from_atomics(
                        asset_info.query_pool(querier, &contract_addr)?,
                        asset_info.decimals(querier, factory_addr)?.into(),
                    )
                    .map_err(|_| StdError::generic_err("Decimal256RangeExceeded"))?,
                })
            })
            .collect()
    }
}

/// Returns [`PairInfo`] by specified pool address.
///
/// * **pool_addr** address of the pool.
pub fn pair_info_by_pool(querier: &QuerierWrapper, pool: impl Into<String>) -> StdResult<PairInfo> {
    let minter_info: MinterResponse = querier.query_wasm_smart(pool, &Cw20QueryMsg::Minter {})?;

    let pair_info: PairInfo =
        querier.query_wasm_smart(minter_info.minter, &PairQueryMsg::Pair {})?;

    Ok(pair_info)
}
